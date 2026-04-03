//! Module de gestion des joueurs du jeu de duel.
//!
//! Ce module contient la structure [`Player`] représentant un joueur
//! avec ses caractéristiques (nom, vitalité, vitesse, force) ainsi
//! que les méthodes associées pour les manipuler.

use log::{debug, info, warn};
use std::fmt;

/// Valeur par défaut de la vitalité d'un joueur.
const DEFAULT_VITALITY: i32 = 100;

/// Valeur par défaut de la vitesse d'un joueur (en ms par incrément du compteur).
const DEFAULT_SPEED: i32 = 50;

/// Valeur par défaut de la force d'un joueur.
const DEFAULT_STRENGTH: i32 = 50;

/// Représente un joueur du jeu de duel.
///
/// Chaque joueur possède un nom, une vitalité, une vitesse et une force.
/// Ces caractéristiques peuvent évoluer au cours de la partie via les
/// poisons appliqués par l'adversaire ou la perte de vitalité en fin de manche.
#[derive(Debug, Clone)]
pub struct Player {
    /// Nom du joueur.
    name: String,
    /// Points de vie du joueur. La partie se termine quand cette valeur atteint 0.
    vitality: i32,
    /// Vitesse du compteur en millisecondes par incrément.
    speed: i32,
    /// Force du joueur, ajoutée au score de base à chaque objectif.
    strength: i32,
}

/// Type de poison applicable à un joueur adverse.
///
/// Le gagnant d'une manche peut choisir d'appliquer l'un de ces
/// deux poisons au perdant pour les manches suivantes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Poison {
    /// Réduit la vitesse du joueur de 5 (augmente le délai du compteur).
    Speed,
    /// Réduit la force du joueur de 5.
    Strength,
}

impl Player {
    /// Crée un nouveau joueur avec les caractéristiques spécifiées.
    ///
    /// # Arguments
    ///
    /// * `name` - Nom du joueur.
    /// * `vitality` - Points de vie initiaux (utilise la valeur par défaut si `None`).
    /// * `speed` - Vitesse du compteur en ms (utilise la valeur par défaut si `None`).
    /// * `strength` - Force du joueur (utilise la valeur par défaut si `None`).
    ///
    /// # Exemples
    ///
    /// ```
    /// let player = Player::new("Michel".to_string(), Some(50), None, None);
    /// assert_eq!(player.name(), "Michel");
    /// assert_eq!(player.vitality(), 50);
    /// ```
    pub fn new(name: String, vitality: Option<i32>, speed: Option<i32>, strength: Option<i32>) -> Self {
        let player = Self {
            name,
            vitality: vitality.unwrap_or(DEFAULT_VITALITY),
            speed: speed.unwrap_or(DEFAULT_SPEED),
            strength: strength.unwrap_or(DEFAULT_STRENGTH),
        };
        info!(
            "Joueur créé : {} (V={}, S={}, F={})",
            player.name, player.vitality, player.speed, player.strength
        );
        player
    }

    /// Retourne le nom du joueur.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Retourne la vitalité actuelle du joueur.
    pub fn vitality(&self) -> i32 {
        self.vitality
    }

    /// Retourne la vitesse actuelle du joueur (en ms par incrément).
    pub fn speed(&self) -> i32 {
        self.speed
    }

    /// Retourne la force actuelle du joueur.
    pub fn strength(&self) -> i32 {
        self.strength
    }

    /// Vérifie si le joueur est encore en vie (vitalité > 0).
    ///
    /// # Retourne
    ///
    /// `true` si la vitalité du joueur est strictement positive, `false` sinon.
    pub fn is_alive(&self) -> bool {
        self.vitality > 0
    }

    /// Inflige des dégâts au joueur en réduisant sa vitalité.
    ///
    /// La vitalité ne descend pas en dessous de 0.
    ///
    /// # Arguments
    ///
    /// * `damage` - Nombre de points de vitalité à retirer.
    pub fn take_damage(&mut self, damage: i32) {
        self.vitality = (self.vitality - damage).max(0);
        warn!(
            "{} subit {} dégâts. Vitalité restante : {}",
            self.name, damage, self.vitality
        );
    }

    /// Applique un poison au joueur, réduisant l'une de ses caractéristiques.
    ///
    /// # Arguments
    ///
    /// * `poison` - Le type de poison à appliquer ([`Poison::Speed`] ou [`Poison::Strength`]).
    pub fn apply_poison(&mut self, poison: Poison) {
        match poison {
            Poison::Speed => {
                self.speed = (self.speed + 5).max(5);
                debug!(
                    "Poison vitesse appliqué à {}. Nouvelle vitesse : {} ms",
                    self.name, self.speed
                );
            }
            Poison::Strength => {
                self.strength = (self.strength - 5).max(0);
                debug!(
                    "Poison force appliqué à {}. Nouvelle force : {}",
                    self.name, self.strength
                );
            }
        }
        info!(
            "Poison {:?} appliqué à {}",
            poison, self.name
        );
    }

    /// Réinitialise les caractéristiques du joueur pour une nouvelle partie.
    ///
    /// # Arguments
    ///
    /// * `vitality` - Nouvelle vitalité initiale.
    /// * `speed` - Nouvelle vitesse initiale (optionnelle, garde la valeur par défaut sinon).
    /// * `strength` - Nouvelle force initiale (optionnelle, garde la valeur par défaut sinon).
    pub fn reset(&mut self, vitality: i32, speed: Option<i32>, strength: Option<i32>) {
        self.vitality = vitality;
        self.speed = speed.unwrap_or(DEFAULT_SPEED);
        self.strength = strength.unwrap_or(DEFAULT_STRENGTH);
        info!("{} réinitialisé pour une nouvelle partie.", self.name);
    }
}

/// Implémentation de l'affichage formaté pour un joueur.
///
/// Produit une sortie du type : `Michel (Vitality=50, Speed=50, Strength=50)`
impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (Vitality={}, Speed={}, Strength={})",
            self.name, self.vitality, self.speed, self.strength
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Teste la création d'un joueur avec des valeurs par défaut.
    #[test]
    fn test_new_player_defaults() {
        let player = Player::new("Test".to_string(), None, None, None);
        assert_eq!(player.name(), "Test");
        assert_eq!(player.vitality(), DEFAULT_VITALITY);
        assert_eq!(player.speed(), DEFAULT_SPEED);
        assert_eq!(player.strength(), DEFAULT_STRENGTH);
    }

    /// Teste la création d'un joueur avec des valeurs personnalisées.
    #[test]
    fn test_new_player_custom() {
        let player = Player::new("Alice".to_string(), Some(80), Some(30), Some(60));
        assert_eq!(player.vitality(), 80);
        assert_eq!(player.speed(), 30);
        assert_eq!(player.strength(), 60);
    }

    /// Teste que le joueur est en vie quand sa vitalité est positive.
    #[test]
    fn test_is_alive_true() {
        let player = Player::new("Alive".to_string(), Some(1), None, None);
        assert!(player.is_alive());
    }

    /// Teste que le joueur est mort quand sa vitalité est à 0.
    #[test]
    fn test_is_alive_false() {
        let player = Player::new("Dead".to_string(), Some(0), None, None);
        assert!(!player.is_alive());
    }

    /// Teste l'application de dégâts au joueur.
    #[test]
    fn test_take_damage() {
        let mut player = Player::new("Test".to_string(), Some(50), None, None);
        player.take_damage(13);
        assert_eq!(player.vitality(), 37);
    }

    /// Teste que la vitalité ne descend pas en dessous de 0.
    #[test]
    fn test_take_damage_no_negative() {
        let mut player = Player::new("Test".to_string(), Some(10), None, None);
        player.take_damage(20);
        assert_eq!(player.vitality(), 0);
        assert!(!player.is_alive());
    }

    /// Teste l'application du poison de vitesse.
    #[test]
    fn test_apply_poison_speed() {
        let mut player = Player::new("Test".to_string(), None, Some(50), None);
        player.apply_poison(Poison::Speed);
        // Speed poison increases the delay (slower counter)
        assert_eq!(player.speed(), 55);
    }

    /// Teste l'application du poison de force.
    #[test]
    fn test_apply_poison_strength() {
        let mut player = Player::new("Test".to_string(), None, None, Some(50));
        player.apply_poison(Poison::Strength);
        assert_eq!(player.strength(), 45);
    }

    /// Teste que la force ne descend pas en dessous de 0.
    #[test]
    fn test_apply_poison_strength_min_zero() {
        let mut player = Player::new("Test".to_string(), None, None, Some(3));
        player.apply_poison(Poison::Strength);
        assert_eq!(player.strength(), 0);
    }

    /// Teste la réinitialisation du joueur.
    #[test]
    fn test_reset() {
        let mut player = Player::new("Test".to_string(), Some(10), Some(80), Some(20));
        player.reset(100, None, None);
        assert_eq!(player.vitality(), 100);
        assert_eq!(player.speed(), DEFAULT_SPEED);
        assert_eq!(player.strength(), DEFAULT_STRENGTH);
    }

    /// Teste l'affichage formaté du joueur.
    #[test]
    fn test_display() {
        let player = Player::new("Michel".to_string(), Some(50), Some(50), Some(50));
        let display = format!("{}", player);
        assert_eq!(display, "Michel (Vitality=50, Speed=50, Strength=50)");
    }
}
