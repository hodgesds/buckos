use serde::Deserialize;
// use url::Url;

#[derive(Deserialize)]
pub struct Processor {
    architecture: String,
    //op_modes: String, 32/64 bit
    // address_sizes: 48 bits physical, 48 bits virtual
    // byte_order: String, little endian
    cpus: u16,
    online_cpus: String,
    vendor_id: String,
    model_name: String,
    cpu_family: String,
    model: String,
    threads_per_core: u8,
    cores_per_socket: u16,
    sockets: u8,
    stepping: String,
    freq_boost: String,       // TODO
    freq_scaling_mhz: String, // TODO
    cpu_max_mhz: u32,
    cpu_min_mhz: u32,
    bogo_mips: f64,
    flags: String,
    virtualization: String,
    l1d_cache: String,
    l1i_cache: String,
    l2_cache: String,
    l3_cache: String,
    numa_nodes: u8,
    // numa_node_cpus: Vec, // TODO
}

/*

[[lscpu]]
field = "Vulnerability Itlb multihit:"
data = "Not affected"

[[lscpu]]
field = "Vulnerability L1tf:"
data = "Not affected"

[[lscpu]]
field = "Vulnerability Mds:"
data = "Not affected"

[[lscpu]]
field = "Vulnerability Meltdown:"
data = "Not affected"

[[lscpu]]
field = "Vulnerability Mmio stale data:"
data = "Not affected"

[[lscpu]]
field = "Vulnerability Retbleed:"
data = "Mitigation; untrained return thunk; SMT enabled with STIBP protection"

[[lscpu]]
field = "Vulnerability Spec store bypass:"
data = "Mitigation; Speculative Store Bypass disabled via prctl"

[[lscpu]]
field = "Vulnerability Spectre v1:"
data = "Mitigation; usercopy/swapgs barriers and __user pointer sanitization"

[[lscpu]]
field = "Vulnerability Spectre v2:"
data = "Mitigation; Retpolines, IBPB conditional, STIBP always-on, RSB filling, PBRSB-eIBRS Not affected"

[[lscpu]]
field = "Vulnerability Srbds:"
data = "Not affected"

[[lscpu]]
field = "Vulnerability Tsx async abort:"
data = "Not affected"

*/
