use display_info;

fn main() {
    match display_info::DisplayInfo::all() {
        Ok(displays) => {
            println!("Found {} displays:", displays.len());
            for (index, display) in displays.iter().enumerate() {
                println!("  Display {}: ID={}, Scale={}, Width={}, Height={}", 
                    index, display.id, display.scale_factor, display.width, display.height);
            }
        }
        Err(e) => {
            println!("Error getting display info: {}", e);
        }
    }
}
