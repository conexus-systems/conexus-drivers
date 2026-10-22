//! Lê o sysfs real e imprime zonas térmicas e fontes de energia. Só leitura.
//! Rode: `cargo run -p conexus-driver-generic-linux --example probe`
use conexus_driver_generic_linux::{power, thermal, Component, PowerSupply, Thermal};

fn main() {
    let t = Thermal::default();
    let p = PowerSupply::default();
    println!("térmico disponível: {} | energia disponível: {}", t.available(), p.available());
    match thermal::read_zones(&t.root) {
        Ok(zs) => {
            for z in &zs {
                println!("  zona {} {:<20} {:?} °C", z.index, z.kind, z.celsius());
            }
            if let Some(h) = thermal::hottest(&zs) {
                println!("  mais quente: zona {} -> {:?}", h.index, thermal::Thresholds::default().classify(h.temp_millic.unwrap()));
            }
        }
        Err(e) => println!("  /sys/class/thermal ilegível: {e}"),
    }
    match power::read_supplies(&p.root) {
        Ok(ss) => {
            for s in &ss {
                println!("  {} {:?} {:?} cap={:?} temp={:?}", s.name, s.kind, s.status, s.capacity, s.temp_celsius());
            }
            println!("  energia externa: {:?}", power::external_power(&ss));
        }
        Err(e) => println!("  /sys/class/power_supply ilegível: {e}"),
    }
}
