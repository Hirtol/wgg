use crate::api::pagination::connection::{Connection, Edge};
use crate::api::GraphqlResult;
use async_graphql::connection::{CursorType, PageInfo};
use async_graphql::OutputType;
use std::future::Future;

pub const DEFAULT_PAGE_SIZE: usize = 200;

pub type ConnectionResult<T> = GraphqlResult<Connection<T>>;

pub struct QueryResult<T, I: IntoIterator<Item = T>> {
    pub iter: I,
    pub total_count: u64,
}

/// Creates a new Relay-compliant connection.
/// This assumes offset-pagination is used in the back-end.
///
/// # Arguments
///
/// * `after` - The cursor after which we want to get results.
/// * `first` - The first x items we want, mutually exclusive with `last`
/// * `f` - Function which will return an iterator and optional total row count of the desired object.
pub async fn offset_query<Node: OutputType, F, R, I: IntoIterator<Item = Node>>(
    after: Option<String>,
    first: Option<i32>,
    f: F,
) -> ConnectionResult<Node>
where
    F: FnOnce(Option<WggCursor>, usize) -> R,
    R: Future<Output = GraphqlResult<QueryResult<Node, I>>>,
{
    d_query(after, None, first, None, |offset, _, first, _| {
        f(offset, first.unwrap_or(DEFAULT_PAGE_SIZE))
    })
    .await
}

/// Creates a new Relay-compliant connection.
///
/// # Arguments
///
/// * `after` - The cursor after which we want to get results.
/// * `before` - The cursor before which we want to get results, can interact with `after`
/// * `first` - The first x items we want, mutually exclusive with `last`
/// * `last` - The last x items we want, mutually exclusive with `first`
/// * `f` - Function which will return an iterator and optional total row count of the desired object.
pub async fn d_query<Node: OutputType, F, R, I: IntoIterator<Item = Node>>(
    after: Option<String>,
    before: Option<String>,
    first: Option<i32>,
    last: Option<i32>,
    f: F,
) -> ConnectionResult<Node>
where
    F: FnOnce(Option<WggCursor>, Option<WggCursor>, Option<usize>, Option<usize>) -> R,
    R: Future<Output = GraphqlResult<QueryResult<Node, I>>>,
{
    Ok(async_graphql::connection::query_with(
        after,
        before,
        first,
        last,
        |after: Option<WggCursor>, before, first, last| async move {
            let offset: Option<WggCursor> = after.map(|b| b.increment().into());

            let result = f(offset, before, first, last).await?;

            query(
                result.iter,
                offset,
                before,
                first,
                last,
                DEFAULT_PAGE_SIZE,
                result.total_count as usize,
            )
            .await
        },
    )
    .await?)
}

/// Creates a new Relay-compliant connection.
///
/// # Arguments
///
/// * `iter` - The items to create a paginated collection out of.
/// * `after` - The cursor after which we want to get results.
/// * `before` - The cursor before which we want to get results, can interact with `after`
/// * `first` - The first x items we want, mutually exclusive with `last`
/// * `last` - The last x items we want, mutually exclusive with `first`
/// * `default_page_size` - The default size of one page for this query
/// * `total_count` - The optional total row count for this query, if elided it will take the length of `iter` instead.
pub async fn query<T: OutputType, I: IntoIterator<Item = T>>(
    iter: I,
    after: Option<WggCursor>,
    before: Option<WggCursor>,
    first: Option<usize>,
    last: Option<usize>,
    default_page_size: usize,
    total_count: usize,
) -> ConnectionResult<T> {
    let (start, end) = {
        let after = after.map(|a| a.index()).unwrap_or(0);
        let before: usize = before.map(|b| b.into()).unwrap_or(total_count);

        // Calculate start/end based on the provided first/last. Note that async-graphql disallows
        // providing both (returning an error), so we can safely assume we have, at most, one of
        // first or last.
        match (first, last) {
            // First
            (Some(first), _) => (after, (after.saturating_add(first)).min(before)),
            // Last
            (_, Some(last)) => ((before.saturating_sub(last)).max(after), before),
            // Default page size
            _ => (after, default_page_size.min(before)),
        }
    };

    let edges: Vec<Edge<T>> = (start..end)
        .into_iter()
        .zip(iter)
        .map(|(cursor, node)| connection::Edge::new(cursor.into(), node))
        .collect();

    let connection = connection::Connection::new(
        PageInfo {
            has_previous_page: start > 0,
            has_next_page: end < total_count,
            start_cursor: edges.first().map(|e| e.cursor.encode_cursor()),
            end_cursor: edges.last().map(|e| e.cursor.encode_cursor()),
        },
        total_count as u32,
        edges,
    );

    Ok(connection)
}

mod connection {
    use crate::api::pagination::WggCursor;
    use async_graphql::connection::{CursorType, PageInfo};
    use async_graphql::{Object, OutputType, SimpleObject, TypeName};
    use std::borrow::Cow;

    /// An edge in a connection
    #[derive(SimpleObject)]
    #[graphql(name_type)]
    pub struct Edge<T: OutputType> {
        pub(crate) cursor: String,
        pub(crate) node: T,
    }

    impl<T: OutputType> TypeName for Edge<T> {
        fn type_name() -> Cow<'static, str> {
            format!("{}Edge", T::type_name()).into()
        }
    }

    /// A list of edges
    pub struct Connection<T: OutputType> {
        pub(crate) edges: Vec<Edge<T>>,
        pub(crate) page_info: PageInfo,
        pub(crate) total_count: u32,
    }

    #[Object(name_type)]
    impl<T: OutputType> Connection<T> {
        pub async fn edges(&self) -> &[Edge<T>] {
            &self.edges
        }

        /// A list of nodes.
        pub async fn nodes(&self) -> Vec<&T> {
            self.edges.iter().map(|e| &e.node).collect()
        }

        /// Information about the current page.
        pub async fn page_info(&self) -> &PageInfo {
            &self.page_info
        }

        /// The total amount of items available in this collection
        pub async fn total_count(&self) -> u32 {
            self.total_count
        }
    }

    impl<T: OutputType> TypeName for Connection<T> {
        fn type_name() -> Cow<'static, str> {
            format!("{}Connection", T::type_name()).into()
        }
    }

    impl<T: OutputType> Connection<T> {
        pub fn new(page_info: PageInfo, total_count: u32, edges: Vec<Edge<T>>) -> Self {
            Self {
                edges,
                page_info,
                total_count,
            }
        }
    }

    impl<T: OutputType> Edge<T> {
        pub fn new(cursor: WggCursor, node: T) -> Self {
            Self {
                cursor: cursor.encode_cursor(),
                node,
            }
        }
    }
}

/// Default integer cursor implementation
#[derive(Clone, Copy, Default, Debug)]
pub struct WggCursor(usize);

impl WggCursor {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Increment and return the index. Uses saturating_add to avoid overflow
    /// issues.
    pub const fn increment(&self) -> usize {
        self.0.saturating_add(1)
    }

    pub const fn index(&self) -> usize {
        self.0
    }

    pub const fn offset(&self) -> u64 {
        self.0 as u64
    }

    fn encode(&self) -> String {
        self.0.to_string()
    }

    fn decode(s: &str) -> anyhow::Result<Self> {
        Ok(Self::new(s.parse()?))
    }
}

impl From<WggCursor> for usize {
    fn from(cursor: WggCursor) -> Self {
        cursor.0
    }
}

impl From<WggCursor> for u64 {
    fn from(cursor: WggCursor) -> Self {
        cursor.0 as u64
    }
}

impl From<usize> for WggCursor {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl From<u64> for WggCursor {
    fn from(index: u64) -> Self {
        Self(index as usize)
    }
}

impl From<i32> for WggCursor {
    fn from(index: i32) -> Self {
        Self(index as usize)
    }
}

impl From<WggCursor> for i32 {
    fn from(cursor: WggCursor) -> Self {
        cursor.0 as i32
    }
}

impl CursorType for WggCursor {
    type Error = anyhow::Error;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        WggCursor::decode(s)
    }

    fn encode_cursor(&self) -> String {
        self.encode()
    }
}
