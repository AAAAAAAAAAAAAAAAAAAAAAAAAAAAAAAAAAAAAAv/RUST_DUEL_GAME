//! # Jeu de Duel
//!
//! Mini jeu de duel au tour par tour en Rust.
//!
//! Ce programme oppose deux joueurs disposant de caractéristiques propres
//! (nom, vitalité, vitesse, force). À chaque manche, des objectifs aléatoires
//! sont générés et les joueurs doivent arrêter un compteur au plus près de
//! ces objectifs en appuyant sur la touche ENTRÉE.
//!
//! ## Utilisation
//!
//! ```bash
//! cargo run -- --name1 Michel --name2 Jacque --vitality 50 --objectifs 5
//! ```
//!
//! ## Logging
//!
//! Les niveaux de log sont configurables via la variable d'environnement `RUST_LOG` :
//!
//! ```bash
//! RUST_LOG=debug cargo run -- --name1 Michel --name2 Jacque
//! ```

pub mod counter;
pub mod game;
pub mod player;
pub mod scoring;

use clap::Parser;
use game::{Game, GameConfig};
use log::{error, info};

/// Arguments en ligne de commande pour le jeu de duel.
///
/// Utilise la crate `clap` pour parser automatiquement les arguments
/// avec des valeurs par défaut.
#[derive(Parser, Debug)]
#[command(
    name = "duel_game",
    about = "Mini jeu de duel au tour par tour en Rust",
    version = "1.0",
    author = "Étudiant Centrale Nantes"
)]
struct Args {
    /// Nom du premier joueur.
    #[arg(long = "name1", default_value = "Joueur1")]
    name1: String,

    /// Nom du second joueur.
    #[arg(long = "name2", default_value = "Joueur2")]
    name2: String,

    /// Vitalité initiale des joueurs.
    #[arg(long = "vitality", default_value_t = 100)]
    vitality: i32,

    /// Vitesse initiale des joueurs en ms par incrément du compteur.
    #[arg(long = "speed", default_value_t = 50)]
    speed: i32,

    /// Force initiale des joueurs.
    #[arg(long = "strength", default_value_t = 50)]
    strength: i32,

    /// Nombre d'objectifs par manche.
    #[arg(long = "objectifs", default_value_t = 5)]
    objectifs: usize,
}

/// Point d'entrée du programme.
///
/// Parse les arguments de la ligne de commande, initialise le logger,
/// crée la configuration de la partie et lance le jeu.
///
/// # Retourne
///
/// * `Ok(())` si le programme s'est exécuté sans erreur.
/// * `Err` en cas d'erreur fatale (configuration invalide, erreur d'I/O).
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialiser le logger
    env_logger::init();
    info!("Démarrage du jeu de duel.");

    // Parser les arguments
    let args = Args::parse();
    info!(
        "Arguments : name1={}, name2={}, vitality={}, speed={}, strength={}, objectifs={}",
        args.name1, args.name2, args.vitality, args.speed, args.strength, args.objectifs
    );

    // Créer la configuration
    let config = match GameConfig::new(
        args.vitality,
        Some(args.speed),
        Some(args.strength),
        args.objectifs,
    ) {
        Ok(config) => config,
        Err(e) => {
            error!("Configuration invalide : {}", e);
            eprintln!("Erreur de configuration : {}", e);
            return Err(e.into());
        }
    };

    // Créer et lancer la partie
    let mut game = Game::new(args.name1, args.name2, config);

    if let Err(e) = game.run() {
        error!("Erreur durant la partie : {}", e);
        // S'assurer que le mode raw est désactivé en cas d'erreur
        let _ = crossterm::terminal::disable_raw_mode();
        eprintln!("Erreur : {}", e);
        return Err(e);
    }

    info!("Fin du programme.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Teste le parsing d'arguments par défaut.
    #[test]
    fn test_default_args() {
        let args = Args::parse_from(["duel_game"]);
        assert_eq!(args.name1, "Joueur1");
        assert_eq!(args.name2, "Joueur2");
        assert_eq!(args.vitality, 100);
        assert_eq!(args.speed, 50);
        assert_eq!(args.strength, 50);
        assert_eq!(args.objectifs, 5);
    }

    /// Teste le parsing d'arguments personnalisés.
    #[test]
    fn test_custom_args() {
        let args = Args::parse_from([
            "duel_game",
            "--name1", "Michel",
            "--name2", "Jacque",
            "--vitality", "50",
            "--objectifs", "3",
        ]);
        assert_eq!(args.name1, "Michel");
        assert_eq!(args.name2, "Jacque");
        assert_eq!(args.vitality, 50);
        assert_eq!(args.objectifs, 3);
    }

    /// Teste la création de la configuration depuis les arguments.
    #[test]
    fn test_config_from_args() {
        let config = GameConfig::new(50, Some(50), Some(50), 5);
        assert!(config.is_ok());
    }
}
