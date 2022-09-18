use async_graphql::InputType;
use itertools::Itertools;
use sea_orm::Condition;

/// Recursively apply all of the provided filter's conditions, given the provided `filter_func`.
///
/// The `filter_func` only has to handle the inner type's filter items and apply them to the given condition, `and` and `or`
/// conditions are handled elsewhere
///
/// # Errors
///
/// Will return an `Err` if a given filter has a nesting depth greater than `3`, or more than `10` filters in a given condition.
/// This means that the theoretical maximum amount of filters a user can apply is 2000 (1000 for `or` + 1000 for `and`).
///
/// This limit is in place to hopefully prevent DOS attacks.
///
/// # Examples
/// ```
/// # use crate::cross_system::{Filter, recursive_filter}
/// #[derive(async_graphql::InputType)]
/// struct HelloConditions {
///     pub name: String,
/// }
///
/// let conditions = Filter::new(HelloConditions {name: "Hello".to_string()});
/// let filters = recursive_filter(conditions, |cond, hello| cond.add(Columns::Name.eq(&hello.name)));
/// ```
pub fn recursive_filter<T: InputType>(
    root_filter: Filter<T>,
    filter_func: impl Fn(Condition, T) -> Condition,
) -> anyhow::Result<Condition> {
    fn inner<T: InputType>(
        filter: Filter<T>,
        filter_func: &impl Fn(Condition, T) -> Condition,
        depth: usize,
    ) -> anyhow::Result<Condition> {
        // Ensure the filter isn't excessively deep
        if depth > 3 {
            anyhow::bail!("Filters are not allowed to be nested more than 3 times");
        }

        if filter.or.as_ref().map(Vec::len).unwrap_or_default() > 10
            || filter.and.as_ref().map(Vec::len).unwrap_or_default() > 10
        {
            anyhow::bail!("More than 10 filters in one condition is not allowed");
        }

        let mut conditions = Condition::all();

        if let Some(or) = filter.or {
            let or_condition = or
                .into_iter()
                .map(|filter| inner(*filter, filter_func, depth + 1))
                .fold_ok(Condition::any(), |fold, filter| fold.add(filter))?;

            conditions = conditions.add(or_condition);
        }
        if let Some(and) = filter.and {
            let and_condition = and
                .into_iter()
                .map(|filter| inner(*filter, filter_func, depth + 1))
                .fold_ok(Condition::all(), |fold, filter| fold.add(filter))?;

            conditions = conditions.add(and_condition);
        }

        Ok(filter_func(conditions, filter.fields))
    }

    inner(root_filter, &filter_func, 0)
}

/// A [Filter] is a recursive filter type which can be used to apply complex filtering for GraphQL queries.
///
/// Please refer to [recursive_filter] to apply such a filter smoothly.
#[derive(Debug, Clone, Default)]
pub struct Filter<T: InputType> {
    pub or: Option<Vec<Box<Filter<T>>>>,
    pub and: Option<Vec<Box<Filter<T>>>>,
    pub fields: T,
}

impl<T: InputType> Filter<T> {
    pub fn new(input: T) -> Self {
        Filter {
            or: None,
            and: None,
            fields: input,
        }
    }

    /// Recursively apply all of the provided filter's conditions, given the provided `filter_func`.
    ///
    /// The `filter_func` only has to handle the inner type's filter items and apply them to the given condition, `and` and `or`
    /// conditions are handled elsewhere
    ///
    /// # Errors
    ///
    /// Will return an `Err` if a given filter has a nesting depth greater than `3`, or more than `10` filters in a given condition.
    /// This means that the theoretical maximum amount of filters a user can apply is 2000 (1000 for `or` + 1000 for `and`).
    ///
    /// This limit is in place to hopefully prevent DOS attacks.
    ///
    /// # Examples
    /// ```
    /// # use crate::cross_system::{Filter, recursive_filter}
    /// #[derive(async_graphql::InputType)]
    /// struct HelloConditions {
    ///     pub name: String,
    /// }
    ///
    /// let conditions = Filter::new(HelloConditions {name: "Hello".to_string()});
    /// let filters = recursive_filter(conditions, |cond, hello| cond.add(Columns::Name.eq(&hello.name)));
    /// ```
    pub fn apply_recursive(self, filter_func: impl Fn(Condition, T) -> Condition) -> anyhow::Result<Condition> {
        recursive_filter(self, filter_func)
    }
}

impl<T: InputType> InputType for Filter<T> {
    type RawValueType = Self;

    fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
        format!("{}Filter", T::type_name()).into()
    }

    fn create_type_info(registry: &mut async_graphql::registry::Registry) -> String {
        registry.create_input_type::<Self, _>(async_graphql::registry::MetaTypeId::InputObject, |registry| {
            async_graphql::registry::MetaType::InputObject {
                name: Self::type_name().into_owned(),
                description: None,
                input_fields: {
                    let mut fields = async_graphql::indexmap::IndexMap::new();
                    fields.insert(
                        "or".to_string(),
                        async_graphql::registry::MetaInputValue {
                            name: "or",
                            description: None,
                            ty: <Option<Vec<Box<Filter<T>>>> as InputType>::create_type_info(registry),
                            default_value: None,
                            visible: None,
                            inaccessible: false,
                            tags: &[],
                            is_secret: false,
                        },
                    );
                    fields.insert(
                        "and".to_string(),
                        async_graphql::registry::MetaInputValue {
                            name: "and",
                            description: None,
                            ty: <Option<Vec<Box<Filter<T>>>> as InputType>::create_type_info(registry),
                            default_value: None,
                            visible: None,
                            inaccessible: false,
                            tags: &[],
                            is_secret: false,
                        },
                    );

                    <T as InputType>::create_type_info(registry);

                    if let async_graphql::registry::MetaType::InputObject { input_fields, .. } =
                        registry.create_fake_input_type::<T>()
                    {
                        fields.extend(input_fields);
                    }

                    fields
                },
                visible: None,
                inaccessible: false,
                tags: &[],
                rust_typename: ::std::any::type_name::<Self>(),
                oneof: false,
            }
        })
    }

    fn parse(value: Option<async_graphql::Value>) -> async_graphql::InputValueResult<Self> {
        if let Some(async_graphql::Value::Object(obj)) = value {
            let or: Option<Vec<Box<Filter<T>>>> = async_graphql::InputType::parse(obj.get("or").cloned())
                .map_err(async_graphql::InputValueError::propagate)?;

            let and: Option<Vec<Box<Filter<T>>>> = async_graphql::InputType::parse(obj.get("and").cloned())
                .map_err(async_graphql::InputValueError::propagate)?;

            let fields: T = InputType::parse(Some(async_graphql::Value::Object(::std::clone::Clone::clone(&obj))))
                .map_err(async_graphql::InputValueError::propagate)?;

            Ok(Self { or, and, fields })
        } else {
            Err(async_graphql::InputValueError::expected_type(value.unwrap_or_default()))
        }
    }

    fn to_value(&self) -> async_graphql::Value {
        let mut map = async_graphql::indexmap::IndexMap::new();
        map.insert(
            async_graphql::Name::new("or"),
            async_graphql::InputType::to_value(&self.or),
        );
        map.insert(
            async_graphql::Name::new("and"),
            async_graphql::InputType::to_value(&self.and),
        );
        if let async_graphql::Value::Object(values) = InputType::to_value(&self.fields) {
            map.extend(values);
        }
        async_graphql::Value::Object(map)
    }

    fn federation_fields() -> Option<String> {
        let mut res = Vec::new();
        if let Some(fields) = <Vec<Box<Filter<T>>> as InputType>::federation_fields() {
            res.push({
                let res = format!("{} {}", "or", fields);
                res
            });
        } else {
            res.push("or".to_string());
        }
        if let Some(fields) = <Vec<Box<Filter<T>>> as InputType>::federation_fields() {
            res.push({
                let res = format!("{} {}", "and", fields);
                res
            });
        } else {
            res.push("and".to_string());
        }
        if let Some(fields) = <T as InputType>::federation_fields() {
            res.push(format!("{} {}", "fields", fields));
        } else {
            res.push("fields".to_string());
        }
        Some(format!("{{ {} }}", res.join(" ")))
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }
}

impl<T: InputType> async_graphql::InputObjectType for Filter<T> {}
