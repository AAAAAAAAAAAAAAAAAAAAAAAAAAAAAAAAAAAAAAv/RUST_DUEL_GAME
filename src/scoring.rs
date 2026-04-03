//! Module de calcul des scores du jeu de duel.
//!
//! Ce module contient les fonctions de calcul de la distance circulaire
//! entre un objectif et un résultat, le calcul du score par objectif
//! selon le barème, et le calcul du score final moyen arrondi.

use log::debug;

/// Calcule la distance circulaire entre deux valeurs sur un cercle de 0 à 99.
///
/// La distance circulaire prend en compte le fait que 95 et 15 sont distants
/// de 20 (et non 80), car le compteur boucle de 0 à 100.
///
/// # Arguments
///
/// * `a` - Première valeur (0 à 100).
/// * `b` - Deuxième valeur (0 à 100).
///
/// # Retourne
///
/// La distance minimale entre les deux valeurs en tenant compte du bouclage.
///
/// # Exemples
///
/// ```
/// assert_eq!(circular_distance(95, 15), 20);
/// assert_eq!(circular_distance(50, 50), 0);
/// ```
pub fn circular_distance(a: u32, b: u32) -> u32 {
    let diff = a.abs_diff(b);
    let wrap_diff = 100 - diff;
    debug!("Distance circulaire entre {} et {} : min({}, {}) = {}", a, b, diff, wrap_diff, diff.min(wrap_diff));
    diff.min(wrap_diff)
}

/// Calcule le score obtenu pour un objectif donné.
///
/// Le score dépend de la différence absolue (circulaire) entre l'objectif
/// et le compteur arrêté, de la force du joueur et du nombre de miss.
///
/// # Barème
///
/// | Différence | Score de base |
/// |------------|---------------|
/// | 0          | 100           |
/// | 1 à 5      | 80            |
/// | 6 à 10     | 60            |
/// | 11 à 20    | 40            |
/// | 21 à 40    | 20            |
/// | > 40       | 0             |
///
/// Formule : `(base + force) / (miss + 1)`
///
/// # Arguments
///
/// * `difference` - Distance circulaire entre l'objectif et le résultat.
/// * `strength` - Force du joueur.
/// * `miss` - Nombre de fois que le compteur a bouclé.
///
/// # Retourne
///
/// Le score calculé pour cet objectif.
pub fn calculate_score(difference: u32, strength: i32, miss: u32) -> f64 {
    let base = match difference {
        0 => 100,
        1..=5 => 80,
        6..=10 => 60,
        11..=20 => 40,
        21..=40 => 20,
        _ => 0,
    };

    let score = (base as f64 + strength as f64) / (miss as f64 + 1.0);
    debug!(
        "Score: base={}, strength={}, miss={} => ({} + {}) / {} = {:.2}",
        base, strength, miss, base, strength, miss + 1, score
    );
    score
}

/// Calcule le score final d'un tour en faisant la moyenne des scores
/// sur chaque objectif, arrondi à l'entier supérieur.
///
/// # Arguments
///
/// * `scores` - Vecteur des scores obtenus pour chaque objectif.
///
/// # Retourne
///
/// * `Ok(score)` - Le score moyen arrondi à l'entier supérieur.
/// * `Err` - Si la liste de scores est vide.
///
/// # Erreurs
///
/// Retourne une erreur si le vecteur de scores est vide.
pub fn calculate_final_score(scores: &[f64]) -> Result<i32, String> {
    if scores.is_empty() {
        return Err("Impossible de calculer le score moyen : aucun score fourni.".to_string());
    }

    let sum: f64 = scores.iter().sum();
    let average = sum / scores.len() as f64;
    let final_score = average.ceil() as i32;

    debug!(
        "Score final : somme={:.2}, nb={}, moyenne={:.2}, arrondi={}",
        sum,
        scores.len(),
        average,
        final_score
    );

    Ok(final_score)
}

/// Génère un vecteur d'objectifs aléatoires entre 0 et 100.
///
/// # Arguments
///
/// * `count` - Nombre d'objectifs à générer.
///
/// # Retourne
///
/// * `Ok(Vec<u32>)` - Vecteur contenant les objectifs générés.
/// * `Err` - Si le nombre d'objectifs demandé est 0.
pub fn generate_objectives(count: usize) -> Result<Vec<u32>, String> {
    if count == 0 {
        return Err("Le nombre d'objectifs doit être supérieur à 0.".to_string());
    }

    use rand::Rng;
    let mut rng = rand::thread_rng();
    let objectives: Vec<u32> = (0..count).map(|_| rng.gen_range(0..=100)).collect();

    debug!("Objectifs générés : {:?}", objectives);
    Ok(objectives)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Teste la distance circulaire pour des valeurs identiques.
    #[test]
    fn test_circular_distance_same() {
        assert_eq!(circular_distance(50, 50), 0);
    }

    /// Teste la distance circulaire directe (sans bouclage).
    #[test]
    fn test_circular_distance_direct() {
        assert_eq!(circular_distance(30, 40), 10);
    }

    /// Teste la distance circulaire avec bouclage (95 et 15 → 20).
    #[test]
    fn test_circular_distance_wrap() {
        assert_eq!(circular_distance(95, 15), 20);
    }

    /// Teste la distance circulaire symétrique.
    #[test]
    fn test_circular_distance_symmetric() {
        assert_eq!(circular_distance(15, 95), circular_distance(95, 15));
    }

    /// Teste le calcul de score pour une différence de 0.
    #[test]
    fn test_score_exact() {
        let score = calculate_score(0, 50, 0);
        assert!((score - 150.0).abs() < f64::EPSILON);
    }

    /// Teste le calcul de score pour une différence de 1 à 5.
    #[test]
    fn test_score_close() {
        let score = calculate_score(3, 50, 0);
        assert!((score - 130.0).abs() < f64::EPSILON);
    }

    /// Teste le calcul de score pour une différence de 6 à 10.
    #[test]
    fn test_score_medium() {
        let score = calculate_score(8, 50, 0);
        assert!((score - 110.0).abs() < f64::EPSILON);
    }

    /// Teste le calcul de score pour une différence de 11 à 20.
    #[test]
    fn test_score_far() {
        let score = calculate_score(15, 50, 0);
        assert!((score - 90.0).abs() < f64::EPSILON);
    }

    /// Teste le calcul de score pour une différence de 21 à 40.
    #[test]
    fn test_score_very_far() {
        let score = calculate_score(30, 50, 0);
        assert!((score - 70.0).abs() < f64::EPSILON);
    }

    /// Teste le calcul de score pour une différence supérieure à 40.
    #[test]
    fn test_score_miss_range() {
        let score = calculate_score(45, 50, 0);
        assert!((score - 50.0).abs() < f64::EPSILON);
    }

    /// Teste l'impact du miss sur le score.
    #[test]
    fn test_score_with_miss() {
        let score = calculate_score(0, 50, 1);
        // (100 + 50) / (1 + 1) = 75
        assert!((score - 75.0).abs() < f64::EPSILON);
    }

    /// Teste le score de l'exemple du sujet : objectif 50, compteur 36, miss 1, force 50.
    #[test]
    fn test_score_example_subject() {
        let diff = circular_distance(50, 36);
        assert_eq!(diff, 14); // 11..=20
        let score = calculate_score(diff, 50, 1);
        // (40 + 50) / 2 = 45
        assert!((score - 45.0).abs() < f64::EPSILON);
    }

    /// Teste le score de l'exemple : objectif 82, compteur 80, miss 0, force 50.
    #[test]
    fn test_score_example_82_80() {
        let diff = circular_distance(82, 80);
        assert_eq!(diff, 2); // 1..=5
        let score = calculate_score(diff, 50, 0);
        // (80 + 50) / 1 = 130
        assert!((score - 130.0).abs() < f64::EPSILON);
    }

    /// Teste le calcul du score final moyen.
    #[test]
    fn test_final_score() {
        let scores = vec![45.0, 130.0, 130.0, 55.0, 65.0];
        let result = calculate_final_score(&scores).unwrap();
        // Moyenne : (45+130+130+55+65)/5 = 425/5 = 85
        assert_eq!(result, 85);
    }

    /// Teste le score final arrondi à l'entier supérieur.
    #[test]
    fn test_final_score_ceil() {
        let scores = vec![10.0, 20.0, 30.0]; // moyenne = 20.0
        assert_eq!(calculate_final_score(&scores).unwrap(), 20);

        let scores2 = vec![10.0, 20.0, 31.0]; // moyenne = 20.333...
        assert_eq!(calculate_final_score(&scores2).unwrap(), 21);
    }

    /// Teste l'erreur quand la liste de scores est vide.
    #[test]
    fn test_final_score_empty() {
        let scores: Vec<f64> = vec![];
        assert!(calculate_final_score(&scores).is_err());
    }

    /// Teste la génération d'objectifs.
    #[test]
    fn test_generate_objectives_count() {
        let objectives = generate_objectives(5).unwrap();
        assert_eq!(objectives.len(), 5);
        for &obj in &objectives {
            assert!(obj <= 100);
        }
    }

    /// Teste l'erreur quand le nombre d'objectifs est 0.
    #[test]
    fn test_generate_objectives_zero() {
        assert!(generate_objectives(0).is_err());
    }
}
