//! IPC command handlers

use crate::transport::{TransportFactory, TransportConfig};
use crate::IpcManager;

pub mod repository;

/// Setup IPC handlers with proper dependencies
#[cfg(feature = "tauri")]
pub fn setup_ipc_handlers() -> Result<IpcManager, crate::CziError> {
    // Create transport
    let transport = TransportFactory::create_from_config(&TransportConfig::default())?;

    // Create IPC manager
    let mut manager = IpcManager::new(transport);

    // Register handlers
    manager.register_handler("list_repositories", Box::new(
        crate::commands::RepositoryCommandHandler::new()
    ));
    manager.register_handler("add_repository", Box::new(
        crate::commands::RepositoryCommandHandler::new()
    ));
    manager.register_handler("remove_repository", Box::new(
        crate::commands::RepositoryCommandHandler::new()
    ));
    manager.register_handler("sync_repository", Box::new(
        crate::commands::RepositoryCommandHandler::new()
    ));
    manager.register_handler("validate_repository", Box::new(
        crate::commands::RepositoryCommandHandler::new()
    ));

    manager.register_handler("run_analysis", Box::new(
        crate::commands::AnalysisCommandHandler::new()
    ));
    manager.register_handler("get_analysis_status", Box::new(
        crate::commands::AnalysisCommandHandler::new()
    ));
    manager.register_handler("get_analysis_results", Box::new(
        crate::commands::AnalysisCommandHandler::new()
    ));

    manager.register_handler("query_dependencies", Box::new(
        crate::commands::QueryCommandHandler::new()
    ));
    manager.register_handler("get_symbol_info", Box::new(
        crate::commands::QueryCommandHandler::new()
    ));

    Ok(manager)
}

/// Setup IPC handlers without Tauri
#[cfg(not(feature = "tauri"))]
pub fn setup_ipc_handlers() -> Result<IpcManager, crate::CziError> {
    // Create transport
    let transport = TransportFactory::create_from_config(&TransportConfig::default())?;

    // Create IPC manager
    let mut manager = IpcManager::new(transport);

    // Register handlers
    manager.register_handler("list_repositories", Box::new(
        crate::commands::RepositoryCommandHandler::new()
    ));
    manager.register_handler("add_repository", Box::new(
        crate::commands::RepositoryCommandHandler::new()
    ));
    manager.register_handler("remove_repository", Box::new(
        crate::commands::RepositoryCommandHandler::new()
    ));
    manager.register_handler("sync_repository", Box::new(
        crate::commands::RepositoryCommandHandler::new()
    ));
    manager.register_handler("validate_repository", Box::new(
        crate::commands::RepositoryCommandHandler::new()
    ));

    manager.register_handler("run_analysis", Box::new(
        crate::commands::AnalysisCommandHandler::new()
    ));
    manager.register_handler("get_analysis_status", Box::new(
        crate::commands::AnalysisCommandHandler::new()
    ));
    manager.register_handler("get_analysis_results", Box::new(
        crate::commands::AnalysisCommandHandler::new()
    ));

    manager.register_handler("query_dependencies", Box::new(
        crate::commands::QueryCommandHandler::new()
    ));
    manager.register_handler("get_symbol_info", Box::new(
        crate::commands::QueryCommandHandler::new()
    ));

    Ok(manager)
}