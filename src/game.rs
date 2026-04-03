//! Module principal du jeu de duel.
//!
//! Ce module orchestre le déroulement de la partie : les manches,
//! les tours de jeu, le calcul des scores, l'application des poisons
//! et la gestion de fin de partie.

use crate::counter;
use crate::player::{Player, Poison};
use crate::scoring;
use log::{error, info, warn};
use std::io::{self, Write};

/// Configuration d'une partie.
///
/// Regroupe les paramètres initiaux de la partie comme la vitalité
/// de départ et le nombre d'objectifs par manche.
#[derive(Debug, Clone)]
pub struct GameConfig {
    /// Vitalité initiale des joueurs.
    pub initial_vitality: i32,
    /// Vitesse initiale des joueurs (optionnelle).
    pub initial_speed: Option<i32>,
    /// Force initiale des joueurs (optionnelle).
    pub initial_strength: Option<i32>,
    /// Nombre d'objectifs par manche.
    pub num_objectives: usize,
}

impl GameConfig {
    /// Crée une nouvelle configuration de partie.
    ///
    /// # Arguments
    ///
    /// * `initial_vitality` - Vitalité de départ de chaque joueur.
    /// * `initial_speed` - Vitesse initiale (optionnelle, valeur par défaut utilisée sinon).
    /// * `initial_strength` - Force initiale (optionnelle, valeur par défaut utilisée sinon).
    /// * `num_objectives` - Nombre d'objectifs par manche.
    ///
    /// # Retourne
    ///
    /// * `Ok(GameConfig)` si les paramètres sont valides.
    /// * `Err` si la vitalité est <= 0 ou le nombre d'objectifs est 0.
    pub fn new(
        initial_vitality: i32,
        initial_speed: Option<i32>,
        initial_strength: Option<i32>,
        num_objectives: usize,
    ) -> Result<Self, String> {
        if initial_vitality <= 0 {
            return Err("La vitalité initiale doit être supérieure à 0.".to_string());
        }
        if num_objectives == 0 {
            return Err("Le nombre d'objectifs doit être supérieur à 0.".to_string());
        }
        Ok(Self {
            initial_vitality,
            initial_speed,
            initial_strength,
            num_objectives,
        })
    }
}

/// Structure représentant une partie en cours.
///
/// Contient les deux joueurs, la configuration et le numéro de manche.
pub struct Game {
    /// Premier joueur.
    player1: Player,
    /// Second joueur.
    player2: Player,
    /// Configuration de la partie.
    config: GameConfig,
    /// Numéro de la manche en cours.
    round_number: u32,
}

impl Game {
    /// Crée une nouvelle partie avec les deux joueurs et la configuration.
    ///
    /// # Arguments
    ///
    /// * `name1` - Nom du premier joueur.
    /// * `name2` - Nom du second joueur.
    /// * `config` - Configuration de la partie.
    pub fn new(name1: String, name2: String, config: GameConfig) -> Self {
        let player1 = Player::new(
            name1,
            Some(config.initial_vitality),
            config.initial_speed,
            config.initial_strength,
        );
        let player2 = Player::new(
            name2,
            Some(config.initial_vitality),
            config.initial_speed,
            config.initial_strength,
        );

        Self {
            player1,
            player2,
            config,
            round_number: 0,
        }
    }

    /// Lance la boucle de jeu principale.
    ///
    /// Exécute les manches jusqu'à ce qu'un joueur soit éliminé,
    /// puis propose de relancer une partie.
    ///
    /// # Retourne
    ///
    /// * `Ok(())` si la partie s'est déroulée sans erreur.
    /// * `Err` en cas d'erreur d'I/O ou de logique.
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            self.reset_for_new_game();
            println!("\n##### Démarrage de la partie #####");
            info!("Nouvelle partie démarrée.");

            // Boucle des manches
            while self.player1.is_alive() && self.player2.is_alive() {
                self.round_number += 1;
                println!("\n## Manche {} ##", self.round_number);
                info!("Début de la manche {}.", self.round_number);

                // Tour du joueur 1
                let score1 = self.play_turn(&self.player1.clone())?;
                if !self.player1.is_alive() || !self.player2.is_alive() {
                    break;
                }

                // Tour du joueur 2
                let score2 = self.play_turn(&self.player2.clone())?;

                // Résolution de la manche
                self.resolve_round(score1, score2)?;

                println!("## FIN Manche {} ##", self.round_number);
            }

            // Fin de partie
            println!("\n##### Partie terminée #####");
            if self.player1.is_alive() {
                println!(
                    "{} remporte la partie !",
                    self.player1.name()
                );
            } else {
                println!(
                    "{} remporte la partie !",
                    self.player2.name()
                );
            }

            // Relancer ?
            print!("Relancer une partie ? [Y/N]\n>");
            io::stdout().flush()?;
            match counter::read_yes_no() {
                Ok(true) => {
                    info!("Relance de la partie.");
                    continue;
                }
                _ => {
                    println!("Merci d'avoir joué !");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Exécute le tour d'un joueur : génère les objectifs, lance le compteur,
    /// calcule les scores.
    ///
    /// # Arguments
    ///
    /// * `player` - Référence au joueur dont c'est le tour.
    ///
    /// # Retourne
    ///
    /// * `Ok(i32)` - Le score moyen du tour.
    /// * `Err` en cas d'erreur.
    fn play_turn(&self, player: &Player) -> Result<i32, Box<dyn std::error::Error>> {
        println!("\nAu tour de {}", player);

        // Générer les objectifs
        let objectives = scoring::generate_objectives(self.config.num_objectives)?;
        println!("→ Objectifs : {:?}", objectives);

        // Attendre que le joueur soit prêt
        println!("→ Appuyer sur ENTREE pour démarrer le tour..");
        counter::wait_for_enter()?;

        // Lancer le compteur pour chaque objectif
        let results =
            counter::run_counter_for_objectives(&objectives, player.speed() as u64)?;

        // Calculer les scores pour chaque objectif
        let mut scores = Vec::new();
        for (i, (&objective, result)) in objectives.iter().zip(results.iter()).enumerate() {
            let diff = scoring::circular_distance(objective, result.counter_value);
            let score = scoring::calculate_score(diff, player.strength(), result.miss_count);

            println!(
                "→ Objectif {} : Miss = {} | Compteur = {} // Score = {:.0}",
                objective, result.miss_count, result.counter_value, score
            );

            info!(
                "Objectif {}/{}: cible={}, compteur={}, miss={}, diff={}, score={:.2}",
                i + 1,
                objectives.len(),
                objective,
                result.counter_value,
                result.miss_count,
                diff,
                score
            );

            scores.push(score);
        }

        // Calculer le score final
        let final_score = scoring::calculate_final_score(&scores)?;
        println!("# Fin du tour #");
        println!("→ Score moyen {}", final_score);

        info!(
            "Fin du tour de {}. Score moyen : {}",
            player.name(),
            final_score
        );

        Ok(final_score)
    }

    /// Résout une manche en comparant les scores des deux joueurs.
    ///
    /// Le perdant subit des dégâts égaux à la différence des scores,
    /// et le gagnant choisit un poison à appliquer.
    ///
    /// # Arguments
    ///
    /// * `score1` - Score du joueur 1.
    /// * `score2` - Score du joueur 2.
    ///
    /// # Retourne
    ///
    /// * `Ok(())` si la résolution s'est déroulée sans erreur.
    /// * `Err` en cas d'erreur d'I/O.
    fn resolve_round(
        &mut self,
        score1: i32,
        score2: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if score1 == score2 {
            println!("\nÉgalité ! Aucun joueur ne perd de vitalité.");
            warn!("Manche {} : égalité ({} - {}).", self.round_number, score1, score2);
            return Ok(());
        }

        let (winner_name, loser_name, damage) = if score1 > score2 {
            let damage = score1 - score2;
            self.player2.take_damage(damage);
            (
                self.player1.name().to_string(),
                self.player2.name().to_string(),
                damage,
            )
        } else {
            let damage = score2 - score1;
            self.player1.take_damage(damage);
            (
                self.player2.name().to_string(),
                self.player1.name().to_string(),
                damage,
            )
        };

        println!(
            "\n{} gagne la manche. {} perd {} points de vitalité.",
            winner_name, loser_name, damage
        );

        // Vérifier si le perdant est encore en vie avant le poison
        let loser_alive = if score1 > score2 {
            self.player2.is_alive()
        } else {
            self.player1.is_alive()
        };

        if !loser_alive {
            info!(
                "{} est éliminé ! Pas de poison à appliquer.",
                loser_name
            );
            return Ok(());
        }

        // Choix du poison
        println!(
            "{} vous devez choisir quel poison appliquer à {} :",
            winner_name, loser_name
        );
        println!("→ 1: -5 speed");
        println!("→ 2: -5 strength");
        print!(">");
        io::stdout().flush()?;

        let poison = loop {
            match counter::read_choice() {
                Ok(1) => break Poison::Speed,
                Ok(2) => break Poison::Strength,
                _ => {
                    error!("Choix invalide, veuillez entrer 1 ou 2.");
                    println!("Choix invalide. Entrez 1 ou 2 :");
                    print!(">");
                    io::stdout().flush()?;
                }
            }
        };

        // Appliquer le poison au perdant
        if score1 > score2 {
            self.player2.apply_poison(poison);
        } else {
            self.player1.apply_poison(poison);
        }

        info!(
            "Poison {:?} appliqué à {} par {}.",
            poison, loser_name, winner_name
        );

        Ok(())
    }

    /// Réinitialise la partie pour un nouveau jeu.
    ///
    /// Remet les caractéristiques des joueurs à leurs valeurs initiales
    /// et le compteur de manches à 0.
    fn reset_for_new_game(&mut self) {
        self.player1.reset(
            self.config.initial_vitality,
            self.config.initial_speed,
            self.config.initial_strength,
        );
        self.player2.reset(
            self.config.initial_vitality,
            self.config.initial_speed,
            self.config.initial_strength,
        );
        self.round_number = 0;
        info!("Partie réinitialisée.");
    }
}

/// Détermine le gagnant entre deux scores.
///
/// # Arguments
///
/// * `score1` - Score du joueur 1.
/// * `score2` - Score du joueur 2.
///
/// # Retourne
///
/// * `Some(1)` si le joueur 1 gagne.
/// * `Some(2)` si le joueur 2 gagne.
/// * `None` en cas d'égalité.
pub fn determine_winner(score1: i32, score2: i32) -> Option<u8> {
    match score1.cmp(&score2) {
        std::cmp::Ordering::Greater => Some(1),
        std::cmp::Ordering::Less => Some(2),
        std::cmp::Ordering::Equal => None,
    }
}

/// Calcule les dégâts subis par le perdant d'une manche.
///
/// # Arguments
///
/// * `winner_score` - Score du gagnant.
/// * `loser_score` - Score du perdant.
///
/// # Retourne
///
/// * `Ok(damage)` - La différence entre les deux scores.
/// * `Err` - Si le score du gagnant n'est pas supérieur à celui du perdant.
pub fn calculate_damage(winner_score: i32, loser_score: i32) -> Result<i32, String> {
    if winner_score <= loser_score {
        return Err("Le score du gagnant doit être supérieur à celui du perdant.".to_string());
    }
    Ok(winner_score - loser_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Teste la création d'une configuration valide.
    #[test]
    fn test_game_config_valid() {
        let config = GameConfig::new(100, None, None, 5);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.initial_vitality, 100);
        assert_eq!(config.num_objectives, 5);
    }

    /// Teste la création d'une configuration avec vitalité invalide.
    #[test]
    fn test_game_config_invalid_vitality() {
        let config = GameConfig::new(0, None, None, 5);
        assert!(config.is_err());
    }

    /// Teste la création d'une configuration avec 0 objectifs.
    #[test]
    fn test_game_config_invalid_objectives() {
        let config = GameConfig::new(100, None, None, 0);
        assert!(config.is_err());
    }

    /// Teste la détermination du gagnant quand le joueur 1 gagne.
    #[test]
    fn test_determine_winner_player1() {
        assert_eq!(determine_winner(95, 82), Some(1));
    }

    /// Teste la détermination du gagnant quand le joueur 2 gagne.
    #[test]
    fn test_determine_winner_player2() {
        assert_eq!(determine_winner(60, 85), Some(2));
    }

    /// Teste l'égalité.
    #[test]
    fn test_determine_winner_draw() {
        assert_eq!(determine_winner(75, 75), None);
    }

    /// Teste le calcul des dégâts.
    #[test]
    fn test_calculate_damage() {
        assert_eq!(calculate_damage(95, 82).unwrap(), 13);
    }

    /// Teste le calcul des dégâts avec un score invalide.
    #[test]
    fn test_calculate_damage_invalid() {
        assert!(calculate_damage(50, 80).is_err());
    }

    /// Teste la création d'une partie.
    #[test]
    fn test_game_creation() {
        let config = GameConfig::new(50, None, None, 5).unwrap();
        let game = Game::new("Michel".to_string(), "Jacque".to_string(), config);
        assert_eq!(game.player1.name(), "Michel");
        assert_eq!(game.player2.name(), "Jacque");
        assert_eq!(game.player1.vitality(), 50);
        assert_eq!(game.round_number, 0);
    }

    /// Teste la réinitialisation d'une partie.
    #[test]
    fn test_reset_for_new_game() {
        let config = GameConfig::new(50, Some(50), Some(50), 5).unwrap();
        let mut game = Game::new("A".to_string(), "B".to_string(), config);
        game.round_number = 5;
        game.player1.take_damage(20);
        game.player1.apply_poison(Poison::Strength);

        game.reset_for_new_game();

        assert_eq!(game.player1.vitality(), 50);
        assert_eq!(game.player1.strength(), 50);
        assert_eq!(game.round_number, 0);
    }

    /// Teste le scénario complet de résolution : vitalité, dégâts.
    #[test]
    fn test_damage_scenario() {
        // Michel score 82, Jacque score 95
        // Jacque gagne, Michel perd 13 points
        let damage = calculate_damage(95, 82).unwrap();
        assert_eq!(damage, 13);

        let mut player = Player::new("Michel".to_string(), Some(50), None, None);
        player.take_damage(damage);
        assert_eq!(player.vitality(), 37);
    }

    /// Teste le scénario de fin de partie (vitalité à 0).
    #[test]
    fn test_game_over_scenario() {
        let mut player = Player::new("Michel".to_string(), Some(37), None, None);
        player.take_damage(40);
        assert_eq!(player.vitality(), 0);
        assert!(!player.is_alive());
    }

    /// Teste la configuration avec des paramètres optionnels.
    #[test]
    fn test_game_config_with_options() {
        let config = GameConfig::new(80, Some(40), Some(60), 3).unwrap();
        assert_eq!(config.initial_vitality, 80);
        assert_eq!(config.initial_speed, Some(40));
        assert_eq!(config.initial_strength, Some(60));
        assert_eq!(config.num_objectives, 3);
    }
}
