use crate::sprotect;
use sysinfo::{System, Disks, Networks};
use wmi::{COMLibrary, WMIConnection, Variant};
use std::collections::HashMap;
use winapi::um::winreg::{HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER};
use winapi::um::winnt::{KEY_READ, REG_SZ};
use winapi::shared::minwindef::HKEY;
use std::ptr;
use std::ffi::OsString;
use std::os::windows::ffi::{OsStringExt, OsStrExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemSpecs {
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
    pub gpu_info: Vec<GpuInfo>,
    pub storage_info: Vec<StorageInfo>,
    pub network_info: Vec<NetworkInfo>,
    pub motherboard_info: MotherboardInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CpuInfo {
    pub name: String,
    pub cores: u32,
    pub threads: u32,
    pub max_speed: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_gb: u64,
    pub stick_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vram_gb: u64,
    pub driver_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageInfo {
    pub drive: String,
    pub total_gb: u64,
    pub available_gb: u64,
    pub file_system: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub name: String,
    pub speed_mbps: u64,
    pub adapter_type: String,
    pub manufacturer: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MotherboardInfo {
    pub manufacturer: String,
    pub model: String,
    pub version: String,
}

pub async fn collect_system_specs() -> Result<SystemSpecs, Box<dyn std::error::Error>> {
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con.into())?;

    let cpu_info = collect_cpu_specs(&wmi_con).await?;
    let memory_info = collect_memory_specs(&wmi_con).await?;
    let gpu_info = collect_gpu_specs(&wmi_con).await?;
    let storage_info = collect_storage_specs().await?;
    let network_info = collect_network_specs(&wmi_con).await?;
    let motherboard_info = collect_motherboard_specs(&wmi_con).await?;

    Ok(SystemSpecs {
        cpu_info,
        memory_info,
        gpu_info,
        storage_info,
        network_info,
        motherboard_info,
    })
}

async fn collect_cpu_specs(wmi_con: &WMIConnection) -> Result<CpuInfo, Box<dyn std::error::Error>> {
    let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(sprotect!("SELECT * FROM Win32_Processor"))?;

    let mut cpu_info = CpuInfo {
        name: sprotect!("Unknown").to_string(),
        cores: 0,
        threads: 0,
        max_speed: 0,
    };

    if let Some(result) = results.first() {
        if let Some(name) = result.get(&sprotect!("Name").to_string()) {
            if let Variant::String(cpu_name) = name {
                cpu_info.name = cpu_name.clone();
            }
        }

        if let Some(cores) = result.get(&sprotect!("NumberOfCores").to_string()) {
            if let Variant::UI4(core_count) = cores {
                cpu_info.cores = *core_count;
            }
        }

        if let Some(threads) = result.get(&sprotect!("NumberOfLogicalProcessors").to_string()) {
            if let Variant::UI4(thread_count) = threads {
                cpu_info.threads = *thread_count;
            }
        }

        if let Some(max_speed) = result.get(&sprotect!("MaxClockSpeed").to_string()) {
            if let Variant::UI4(speed) = max_speed {
                cpu_info.max_speed = *speed;
            }
        }
    }

    Ok(cpu_info)
}

async fn collect_memory_specs(wmi_con: &WMIConnection) -> Result<MemoryInfo, Box<dyn std::error::Error>> {
    let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(sprotect!("SELECT * FROM Win32_PhysicalMemory"))?;

    let mut total_capacity = 0u64;

    for result in &results {
        if let Some(capacity) = result.get(&sprotect!("Capacity").to_string()) {
            if let Variant::UI8(cap) = capacity {
                total_capacity += *cap;
            }
        }
    }

    Ok(MemoryInfo {
        total_gb: total_capacity / 1024 / 1024 / 1024,
        stick_count: results.len(),
    })
}

async fn collect_gpu_specs(wmi_con: &WMIConnection) -> Result<Vec<GpuInfo>, Box<dyn std::error::Error>> {
    let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(sprotect!("SELECT * FROM Win32_VideoController"))?;
    let mut gpus = Vec::new();

    for result in results {
        let mut gpu = GpuInfo {
            name: sprotect!("Unknown").to_string(),
            vram_gb: 0,
            driver_version: sprotect!("Unknown").to_string(),
        };

        if let Some(name) = result.get(&sprotect!("Name").to_string()) {
            if let Variant::String(gpu_name) = name {
                gpu.name = gpu_name.clone();
            }
        }

        if let Some(vram) = result.get(&sprotect!("AdapterRAM").to_string()) {
            if let Variant::UI4(vram_bytes) = vram {
                gpu.vram_gb = *vram_bytes as u64 / 1024 / 1024 / 1024;
            }
        }

        if let Some(driver_version) = result.get(&sprotect!("DriverVersion").to_string()) {
            if let Variant::String(version) = driver_version {
                gpu.driver_version = version.clone();
            }
        }

        gpus.push(gpu);
    }

    Ok(gpus)
}

async fn collect_storage_specs() -> Result<Vec<StorageInfo>, Box<dyn std::error::Error>> {
    let disks = Disks::new_with_refreshed_list();
    let mut storage = Vec::new();

    for disk in &disks {
        storage.push(StorageInfo {
            drive: disk.name().to_string_lossy().to_string(),
            total_gb: disk.total_space() / 1024 / 1024 / 1024,
            available_gb: disk.available_space() / 1024 / 1024 / 1024,
            file_system: disk.file_system().to_string_lossy().to_string(),
        });
    }

    Ok(storage)
}

async fn collect_network_specs(wmi_con: &WMIConnection) -> Result<Vec<NetworkInfo>, Box<dyn std::error::Error>> {
    let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(sprotect!("SELECT * FROM Win32_NetworkAdapter WHERE NetConnectionStatus = 2"))?;
    let mut adapters = Vec::new();

    for result in results {
        let mut adapter = NetworkInfo {
            name: sprotect!("Unknown").to_string(),
            speed_mbps: 0,
            adapter_type: sprotect!("Unknown").to_string(),
            manufacturer: sprotect!("Unknown").to_string(),
        };

        if let Some(name) = result.get(&sprotect!("Name").to_string()) {
            if let Variant::String(adapter_name) = name {
                adapter.name = adapter_name.clone();
            }
        }

        if let Some(speed) = result.get(&sprotect!("Speed").to_string()) {
            if let Variant::UI8(spd) = speed {
                adapter.speed_mbps = *spd / 1_000_000;
            }
        }

        if let Some(adapter_type) = result.get(&sprotect!("AdapterType").to_string()) {
            if let Variant::String(atype) = adapter_type {
                adapter.adapter_type = atype.clone();
            }
        }

        if let Some(manufacturer) = result.get(&sprotect!("Manufacturer").to_string()) {
            if let Variant::String(mfg) = manufacturer {
                adapter.manufacturer = mfg.clone();
            }
        }

        adapters.push(adapter);
    }

    Ok(adapters)
}

async fn collect_motherboard_specs(wmi_con: &WMIConnection) -> Result<MotherboardInfo, Box<dyn std::error::Error>> {
    let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(sprotect!("SELECT * FROM Win32_BaseBoard"))?;

    let mut mb_info = MotherboardInfo {
        manufacturer: sprotect!("Unknown").to_string(),
        model: sprotect!("Unknown").to_string(),
        version: sprotect!("Unknown").to_string(),
    };

    if let Some(result) = results.first() {
        if let Some(manufacturer) = result.get(&sprotect!("Manufacturer").to_string()) {
            if let Variant::String(mfg) = manufacturer {
                mb_info.manufacturer = mfg.clone();
            }
        }

        if let Some(product) = result.get(&sprotect!("Product").to_string()) {
            if let Variant::String(prod) = product {
                mb_info.model = prod.clone();
            }
        }

        if let Some(version) = result.get(&sprotect!("Version").to_string()) {
            if let Variant::String(ver) = version {
                mb_info.version = ver.clone();
            }
        }
    }

    Ok(mb_info)
}