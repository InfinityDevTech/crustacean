pub fn distance_formula(source: (u32, u32), target: (u32, u32)) -> u32 {
    let x = source.0 as i32 - target.0 as i32;
    let y = source.1 as i32 - target.1 as i32;
    ((x.pow(2) + y.pow(2)) as f64).sqrt() as u32
}