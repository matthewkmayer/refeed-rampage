# Shared data types

Authoritative source of types used by backend and frontend.

Some features are opted into via cargo flags. This is done because [Rusoto](https://github.com/rusoto/rusoto) and [dynomite](https://github.com/softprops/dynomite) should only be compiled and used on the backend service.

See this project's [Cargo.toml file](Cargo.toml) to see the features available. The [backend Cargo.toml file](../backend/Cargo.toml) shows how the backend opts in to the Rusoto/dynomite behavior. [lib.rs](lib.rs) shows how the feature flags are used:

```rust
#[cfg_attr(feature = "dynamo_bits", derive(Item))]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq, Hash)]
pub struct Meal {
    #[cfg_attr(feature = "dynamo_bits", dynomite(rename = "mealName"))]
    pub name: String,
    #[cfg_attr(feature = "dynamo_bits", dynomite(partition_key))]
    pub id: Uuid,
    pub photos: Option<String>,
    pub description: String,
    pub stars: Option<i32>,
}
```
