//! Die Driver des kernels.
//!
//! Driver kümmern sich um die Steuerung und Abstraktion von externen und internen Geräten
//! - exceptions: exception- und interrupt-handling
//! - memory_controller
//! - power_management: Feine Kontrolle über den Stromverbrauch des Prozessors
//! - serial: Die DBGU für println! und so
//! - sys_timer: Unter anderem für den Timer-Interrupt zuständig
//! - mmu (Memory Management Unit): Teilweise Überschneidungen mit dem memory_controller

pub mod exceptions;
pub mod memory_controller;
pub mod mmu;
pub mod power_management;
pub mod serial;
pub mod sys_timer;

// unused
pub mod led;
