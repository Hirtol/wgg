use serde::{Deserialize, Serialize};

pub trait Id: Sized {
    fn id(&self) -> &str;
    fn from_id(id: impl Into<String>) -> anyhow::Result<Self>;
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[serde(transparent)]
pub struct ProductId(String);

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[serde(transparent)]
pub struct RuntimeId(String);

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[serde(transparent)]
pub struct TabId(String);

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[serde(transparent)]
pub struct PromotionId(String);

macro_rules! impl_types {
    ($($id:ident),+) => {
        $(
            // Dirty quick impl for now, can do actual error handling in the future
            impl Id for $id {
                fn id(&self) -> &str {
                    &self.0
                }

                fn from_id(id: impl Into<String>) -> anyhow::Result<Self> {
                    Ok(Self(id.into()))
                }
            }

            impl AsRef<str> for $id {
                fn as_ref(&self) -> &str {
                    &self.0
                }
            }

            impl std::borrow::Borrow<str> for $id {
                fn borrow(&self) -> &str {
                    &self.0
                }
            }

            impl std::fmt::Display for $id {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "{}", self.0)
                }
            }

            impl std::str::FromStr for $id {
                type Err = anyhow::Error;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    Self::from_id(s)
                }
            }
        )+
    };
}

impl_types!(ProductId, RuntimeId, TabId, PromotionId);
