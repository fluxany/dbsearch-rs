pub fn dot_product(v1: &[f32], v2: &[f32]) -> f32 {
    v1.iter().zip(v2.iter()).map(|(&x, &y)| x * y).sum()
}

pub fn norm(v: &[f32]) -> f32 {
    v.iter().map(|&x| x * x).sum::<f32>().sqrt()
}

pub fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
    if v1.len() != v2.len() || v1.is_empty() || v2.is_empty() {
        return 0.0; // Return 0 if vectors are empty or have different lengths
    }
    
    let dot = dot_product(v1, v2);
    let (norm1, norm2) = (norm(v1), norm(v2));
    if norm1 == 0.0 || norm2 == 0.0 {
        return 0.0; // Return 0 if any of the norms are 0
    }
    
    dot / (norm1 * norm2)
}