#[derive(Debug)]
pub struct OptimizedEncoding {
    pub bytes: Vec<u8>,
    pub quality: u8,
    pub met_target: bool,
}

pub fn optimize<F>(
    target: u64,
    minimum: u8,
    initial: u8,
    mut encode: F,
) -> Result<OptimizedEncoding, String>
where
    F: FnMut(u8) -> Result<Vec<u8>, String>,
{
    let initial = initial.max(minimum);
    let initial_bytes = encode(initial)?;
    if initial_bytes.len() as u64 <= target {
        return Ok(OptimizedEncoding {
            bytes: initial_bytes,
            quality: initial,
            met_target: true,
        });
    }
    let mut low = minimum;
    let mut high = initial.saturating_sub(1);
    let mut best_under: Option<(u8, Vec<u8>)> = None;
    let mut smallest = (initial, initial_bytes);
    for _ in 0..8 {
        if low > high {
            break;
        }
        let quality = low + (high - low) / 2;
        let bytes = encode(quality)?;
        if bytes.len() < smallest.1.len() {
            smallest = (quality, bytes.clone());
        }
        if bytes.len() as u64 <= target {
            best_under = Some((quality, bytes));
            low = quality.saturating_add(1);
        } else {
            if quality == 0 {
                break;
            }
            high = quality - 1;
        }
    }
    let (quality, bytes, met_target) = if let Some((quality, bytes)) = best_under {
        (quality, bytes, true)
    } else {
        (smallest.0, smallest.1, false)
    };
    Ok(OptimizedEncoding {
        bytes,
        quality,
        met_target,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finds_highest_quality_below_target() {
        let result = optimize(600, 20, 90, |q| Ok(vec![0; q as usize * 10])).unwrap();
        assert_eq!(result.quality, 60);
        assert!(result.met_target);
    }
    #[test]
    fn returns_smallest_when_impossible() {
        let result = optimize(10, 20, 80, |q| Ok(vec![0; 100 + q as usize])).unwrap();
        assert!(!result.met_target);
        assert_eq!(result.quality, 20);
    }
}
