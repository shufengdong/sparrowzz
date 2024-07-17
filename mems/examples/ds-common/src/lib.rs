pub mod dyn_topo;
pub mod static_topo;
pub mod tn_input;

// static topo
pub const STATIC_TOPO_DF_NAME: &str = "static_topo";
pub const TERMINAL_DF_NAME: &str = "terminal_cn_dev";
pub const POINT_DF_NAME: &str = "point_terminal_phase";
// dynamic topo
pub const DYN_TOPO_DF_NAME: &str = "dyn_topo";
pub const DEV_TOPO_DF_NAME: &str = "dev_topo";
// impedance matrix
pub const DEV_CONDUCTOR_DF_NAME: &str = "dev_ohm";
// pf input
pub const SHUNT_MEAS_DF_NAME: &str = "shunt_meas";
pub const TN_INPUT_DF_NAME: &str = "tn_input";
// pf nlp
pub const DS_PF_NLP_OBJ: &str = "3phase_pf_obj";
pub const DS_PF_NLP_CONS: &str = "3phase_pf_cons";