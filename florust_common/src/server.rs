use std::result;

use rocket::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum FlorustServerPluginError {
    #[error("Attempted to register data source ID ({0}), but it already exists.")]
    DataSourceAlreadyExists(String),
    #[error("Attempted to access data source ID ({0}), but ID doesn't exist.")]
    DataSourceDoesntExist(String),
    #[error("Attempted to deregister data source ID ({0}), but it already was deregistered")]
    DataSourceAlreadyDeregistered(String),
    #[error("Attempted to access data source manager ({0}), but manager doesn't exist")]
    DataSourceManagerDoesntExist(String),
    #[error("Data source manager failed with error: {0}")]
    DataSourceManager(DataSourceManagerError),
}

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum DataSourceManagerError {
    #[error("DataSourceManager was given invalid data: {0}")]
    InvalidData(String)
}

/// A specialized [`Result`](result::Result) type for [`DataSourceManager`] operations.
/// 
/// This type was made to avoid having to write [`DataSourceManagerError`] repeatedly for return types
/// as they are used widely and repeatedly in both this module (`server_plugin`) and in the `florust_server`
/// crate.
pub type Result<T> = result::Result<T, DataSourceManagerError>;

/// A trait defining a base data source manager. This is a base type that is used the specialized
/// managers below. This type simply serves as a template to define the functionality that all specialized
/// data manager types share.
#[async_trait]
pub trait DataSourceManager<T>: Sync + Send {
    /// Returns the id associated with the data manager.
    fn manager_id(&self) -> &'static str;

    /// Called when a new data source registers itself to the id belonging to the data source manager.
    /// 
    /// Florust will handle keeping track of what data sources are registered to your data source manager's
    /// id, and will never call this method if the data source is already registered. This method only
    /// exists for data source managers who need to perform some sort of operation when a new source is
    /// registered to them, as such, it is perfectly acceptable to leave this implementation as a stub that
    /// just immediately returns `Ok(())`
    /// 
    /// Returns the unit type if no errors occurred, or a [`DataSourceManagerError`] in case of an error.
    async fn register(&self, id: String) -> Result<()>;

    /// Called when a new data source registers itself to the id belonging to the data source manager.
    /// This method is chosen if the data source provided additional info with the registration request.
    /// 
    /// Florust will handle keeping track of what data sources are registered to your data source manager's
    /// id, and will never call this method if the data source is already registered. This method only
    /// exists for data source managers who need to perform some sort of operation when a new source is
    /// registered to them, as such, it is perfectly acceptable to leave this implementation as a stub that
    /// just immediately returns `Ok(())`
    /// 
    /// Returns the unit type if no errors occurred, or a [`DataSourceManagerError`] in case of an error.
    async fn register_with_data(&self, id: String, data: &[u8]) -> Result<()>;

    /// Called when a data source requests to be deregistered from the data source manager.
    /// 
    /// Florust will handle keeping track of what data sources are registered to your data source manager's
    /// id, and will never call this method if the data source was never registered or is already deregistered.
    /// This method only exists for data source managers who need to perform some sort of operation when a new
    /// source is deregistered from them, as such, it is perfectly acceptable to leave this implementation as a
    /// stub that just immediately returns `Ok(())`
    /// 
    /// Returns the unit type if no errors occurred or a [`DataSourceManagerError`] in case of an error.
    async fn deregister(&self, id: &str) -> Result<()>;

    /// Called when a data source requests to be deregistered from the data source manager. This method
    /// is chosen if the data source provided additional info with the deregistration request.
    /// 
    /// Florust will handle keeping track of what data sources are registered to your data source manager's
    /// id, and will never call this method if the data source was never registered or is already deregistered.
    /// This method only exists for data source managers who need to perform some sort of operation when a new
    /// source is deregistered from them, as such, it is perfectly acceptable to leave this implementation as a
    /// stub that just immediately returns `Ok(())`
    /// 
    /// Returns the unit type if no errors occurred, or a [`DataSourceManagerError`] in case of an error.
    async fn deregister_with_data(&self, id: &str, data: &[u8]) -> Result<()>;

    /// Called when a data source has posted an update. Provides the raw data that the data source
    /// has sent to the Florust server.
    /// 
    /// Florust will handle keeping track of what data sources are registered to your data source manager's
    /// id, and will never call this method if the data source was never registered or is already deregistered.
    /// 
    /// Returns the value parsed from the data, or a [`DataSourceManagerError`] in case of an error.
    async fn update_data(&self, id: &str, data: &[u8]) -> Result<T>;
}

/// One of three specialized types of [`DataSourceManager`] that is responsible for producing data of
/// type [`i64`] from data provided by a data source.
pub type IIntegerDataSourceManager = dyn DataSourceManager<i64>;

/// One of three specialized types of [`DataSourceManager`] that is responsible for producing data of
/// type [`u64`] from data provided by a data source.
pub type UIntegerDataSourceManager = dyn DataSourceManager<u64>;

/// One of three specialized types of [`DataSourceManager`] that is responsible for producing data of
/// type [`f64`] from data provided by a data source.
pub type FloatDataSourceManager = dyn DataSourceManager<f64>;

/// A type representing a double boxed trait. This type is double boxed as a boxed trait object is a fat
/// pointer which would be difficult to transport across FFI boundaries. Boxing the box resolves this issue
/// by making it a normal sized pointer.
pub type FFIResult<T> = Box<Result<Box<T>>>;

/// A function that returns a [`FFIBoxTrait`] which contains an [`IIntegerDataSourceManager`].
pub type CreateIIntegerDataSourceManager = unsafe extern "C" fn(Box<Option<toml::map::Map<String, toml::Value>>>) -> FFIResult<IIntegerDataSourceManager>;

/// A function that returns a [`FFIBoxTrait`] which contains an [`UIntegerDataSourceManager`].
pub type CreateUIntegerDataSourceManager = unsafe extern "C" fn(Box<Option<toml::map::Map<String, toml::Value>>>) -> FFIResult<UIntegerDataSourceManager>;

/// A function that returns a [`FFIBoxTrait`] which contains an [`FloatDataSourceManager`].
pub type CreateFloatDataSourceManager = unsafe extern "C" fn(Box<Option<toml::map::Map<String, toml::Value>>>) -> FFIResult<FloatDataSourceManager>;