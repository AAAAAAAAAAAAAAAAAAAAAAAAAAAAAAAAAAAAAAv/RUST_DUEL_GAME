//! Module de gestion du compteur interactif.
//!
//! Ce module implémente le compteur qui s'incrémente de 0 à 100 et
//! que le joueur doit arrêter en appuyant sur ENTRÉE. Il utilise
//! des threads pour l'incrémentation, l'affichage (toutes les 30ms)
//! et la détection de l'appui clavier.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use log::{debug, info, trace};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Résultat d'un arrêt du compteur pour un objectif donné.
///
/// Contient la valeur du compteur au moment de l'appui et le nombre
/// de fois que le compteur a bouclé (miss).
#[derive(Debug, Clone, Copy)]
pub struct CounterResult {
    /// Valeur du compteur au moment de l'arrêt (0 à 99).
    pub counter_value: u32,
    /// Nombre de fois que le compteur a dépassé 100 et est revenu à 0.
    pub miss_count: u32,
}

/// Données partagées entre les threads du compteur.
///
/// Ces données sont protégées par des mutex et des flags atomiques
/// pour permettre un accès concurrent sûr.
struct SharedCounterState {
    /// Valeur courante du compteur (0 à 99).
    counter: Arc<Mutex<u32>>,
    /// Nombre de bouclages du compteur.
    miss: Arc<Mutex<u32>>,
    /// Indique si le compteur doit s'arrêter.
    stopped: Arc<AtomicBool>,
    /// Indique si tous les threads doivent se terminer.
    terminated: Arc<AtomicBool>,
}

impl SharedCounterState {
    /// Crée un nouvel état partagé avec toutes les valeurs initialisées.
    fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
            miss: Arc::new(Mutex::new(0)),
            stopped: Arc::new(AtomicBool::new(false)),
            terminated: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Réinitialise le compteur et le miss pour un nouvel objectif.
    #[cfg(test)]
    fn reset(&self) {
        *self.counter.lock().unwrap() = 0;
        *self.miss.lock().unwrap() = 0;
        self.stopped.store(false, Ordering::SeqCst);
    }
}

/// Exécute le compteur pour un ensemble d'objectifs et retourne les résultats.
///
/// Pour chaque objectif, un compteur s'incrémente de 0 à 100 avec un pas
/// temporel de `speed_ms` millisecondes. Un thread d'affichage met à jour
/// l'affichage toutes les 30ms. Le joueur appuie sur ENTRÉE pour arrêter
/// le compteur.
///
/// # Arguments
///
/// * `objectives` - Liste des objectifs à atteindre.
/// * `speed_ms` - Délai en millisecondes entre chaque incrément du compteur.
///
/// # Retourne
///
/// * `Ok(Vec<CounterResult>)` - Les résultats pour chaque objectif.
/// * `Err` - En cas d'erreur d'I/O ou de terminal.
pub fn run_counter_for_objectives(
    objectives: &[u32],
    speed_ms: u64,
) -> Result<Vec<CounterResult>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    // Activer le mode raw pour détecter les touches sans attendre ENTRÉE
    crossterm::terminal::enable_raw_mode()?;

    for (i, &objective) in objectives.iter().enumerate() {
        let state = SharedCounterState::new();

        // Cloner les références Arc pour le thread compteur
        let counter_clone = Arc::clone(&state.counter);
        let miss_clone = Arc::clone(&state.miss);
        let stopped_clone = Arc::clone(&state.stopped);
        let terminated_clone = Arc::clone(&state.terminated);

        // Thread d'incrémentation du compteur
        let counter_thread = thread::spawn(move || {
            while !stopped_clone.load(Ordering::SeqCst)
                && !terminated_clone.load(Ordering::SeqCst)
            {
                thread::sleep(Duration::from_millis(speed_ms));
                if stopped_clone.load(Ordering::SeqCst) {
                    break;
                }
                let mut counter = counter_clone.lock().unwrap();
                *counter += 1;
                if *counter >= 100 {
                    *counter = 0;
                    let mut miss = miss_clone.lock().unwrap();
                    *miss += 1;
                    trace!("Compteur bouclé, miss = {}", *miss);
                }
            }
        });

        // Cloner pour le thread d'affichage
        let display_counter = Arc::clone(&state.counter);
        let display_miss = Arc::clone(&state.miss);
        let display_stopped = Arc::clone(&state.stopped);
        let display_terminated = Arc::clone(&state.terminated);

        // Thread d'affichage (mise à jour toutes les 30ms)
        let display_thread = thread::spawn(move || {
            while !display_stopped.load(Ordering::SeqCst)
                && !display_terminated.load(Ordering::SeqCst)
            {
                let counter = *display_counter.lock().unwrap();
                let miss = *display_miss.lock().unwrap();
                print!("\r   Compteur = {:>3} | Miss = {}   ", counter, miss);
                let _ = io::stdout().flush();
                thread::sleep(Duration::from_millis(30));
            }
        });

        // Attendre l'appui sur ENTRÉE dans le thread principal
        loop {
            if event::poll(Duration::from_millis(10))? {
                match event::read()? {
                    // CTRL+C : quitter proprement
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        kind: KeyEventKind::Press,
                        ..
                    }) => {
                        state.stopped.store(true, Ordering::SeqCst);
                        let _ = counter_thread.join();
                        state.terminated.store(true, Ordering::SeqCst);
                        let _ = display_thread.join();
                        print!("\r\x1b[K");
                        crossterm::terminal::disable_raw_mode()?;
                        println!("\nPartie interrompue.");
                        std::process::exit(0);
                    }
                    // ENTRÉE : arrêter le compteur
                    Event::Key(KeyEvent {
                        code: KeyCode::Enter,
                        kind: KeyEventKind::Press,
                        ..
                    }) => {
                        state.stopped.store(true, Ordering::SeqCst);
                        break;
                    }
                    _ => {}
                }
            }
        }

        // Attendre la fin des threads
        let _ = counter_thread.join();
        state.terminated.store(true, Ordering::SeqCst);
        let _ = display_thread.join();

        // Récupérer les valeurs finales
        let final_counter = *state.counter.lock().unwrap();
        let final_miss = *state.miss.lock().unwrap();

        let result = CounterResult {
            counter_value: final_counter,
            miss_count: final_miss,
        };

        // Effacer la ligne et afficher le résultat final
        print!("\r\x1b[K");
        info!(
            "Objectif {} : compteur={}, miss={}",
            objective, final_counter, final_miss
        );

        debug!(
            "Objectif {}/{} terminé : counter={}, miss={}",
            i + 1,
            objectives.len(),
            final_counter,
            final_miss
        );

        results.push(result);
    }

    // Désactiver le mode raw
    crossterm::terminal::disable_raw_mode()?;

    Ok(results)
}

/// Attend que le joueur appuie sur ENTRÉE.
///
/// Active le mode raw du terminal pour détecter l'appui immédiatement,
/// puis le désactive après la détection.
///
/// # Retourne
///
/// * `Ok(())` - L'appui a été détecté avec succès.
/// * `Err` - En cas d'erreur d'I/O.
pub fn wait_for_enter() -> Result<(), Box<dyn std::error::Error>> {
    crossterm::terminal::enable_raw_mode()?;
    loop {
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                // CTRL+C : quitter proprement
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    crossterm::terminal::disable_raw_mode()?;
                    println!("\nPartie interrompue.");
                    std::process::exit(0);
                }
                // ENTRÉE : continuer
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    break;
                }
                _ => {}
            }
        }
    }
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

/// Lit un choix utilisateur (1 ou 2) depuis le terminal.
///
/// # Retourne
///
/// * `Ok(1)` ou `Ok(2)` selon le choix du joueur.
/// * `Err` en cas d'erreur de lecture ou de choix invalide après trop de tentatives.
pub fn read_choice() -> Result<u8, Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice: u8 = input.trim().parse().map_err(|_| "Choix invalide")?;
    if choice != 1 && choice != 2 {
        return Err("Le choix doit être 1 ou 2.".into());
    }
    Ok(choice)
}

/// Lit une réponse Y/N depuis le terminal.
///
/// # Retourne
///
/// * `Ok(true)` si le joueur a répondu Y ou y.
/// * `Ok(false)` si le joueur a répondu N ou n.
/// * `Err` en cas de réponse invalide.
pub fn read_yes_no() -> Result<bool, Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    match input.trim().to_lowercase().as_str() {
        "y" | "yes" | "o" | "oui" => Ok(true),
        "n" | "no" | "non" => Ok(false),
        _ => Err("Réponse invalide. Entrez Y ou N.".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Teste la création d'un CounterResult.
    #[test]
    fn test_counter_result_creation() {
        let result = CounterResult {
            counter_value: 42,
            miss_count: 1,
        };
        assert_eq!(result.counter_value, 42);
        assert_eq!(result.miss_count, 1);
    }

    /// Teste la copie d'un CounterResult.
    #[test]
    fn test_counter_result_copy() {
        let result = CounterResult {
            counter_value: 50,
            miss_count: 0,
        };
        let copy = result;
        assert_eq!(copy.counter_value, result.counter_value);
        assert_eq!(copy.miss_count, result.miss_count);
    }

    /// Teste l'initialisation de SharedCounterState.
    #[test]
    fn test_shared_state_init() {
        let state = SharedCounterState::new();
        assert_eq!(*state.counter.lock().unwrap(), 0);
        assert_eq!(*state.miss.lock().unwrap(), 0);
        assert!(!state.stopped.load(Ordering::SeqCst));
        assert!(!state.terminated.load(Ordering::SeqCst));
    }

    /// Teste la réinitialisation de SharedCounterState.
    #[test]
    fn test_shared_state_reset() {
        let state = SharedCounterState::new();
        *state.counter.lock().unwrap() = 50;
        *state.miss.lock().unwrap() = 3;
        state.stopped.store(true, Ordering::SeqCst);

        state.reset();

        assert_eq!(*state.counter.lock().unwrap(), 0);
        assert_eq!(*state.miss.lock().unwrap(), 0);
        assert!(!state.stopped.load(Ordering::SeqCst));
    }

    /// Teste le thread d'incrémentation du compteur en isolation.
    #[test]
    fn test_counter_increment_thread() {
        let counter = Arc::new(Mutex::new(0u32));
        let miss = Arc::new(Mutex::new(0u32));
        let stopped = Arc::new(AtomicBool::new(false));

        let c = Arc::clone(&counter);
        let m = Arc::clone(&miss);
        let s = Arc::clone(&stopped);

        let handle = thread::spawn(move || {
            for _ in 0..5 {
                thread::sleep(Duration::from_millis(10));
                if s.load(Ordering::SeqCst) {
                    break;
                }
                let mut counter = c.lock().unwrap();
                *counter += 1;
                if *counter >= 100 {
                    *counter = 0;
                    let mut miss = m.lock().unwrap();
                    *miss += 1;
                }
            }
        });

        handle.join().unwrap();
        let final_val = *counter.lock().unwrap();
        assert_eq!(final_val, 5);
        assert_eq!(*miss.lock().unwrap(), 0);
    }

    /// Teste le bouclage du compteur (miss).
    #[test]
    fn test_counter_wrap_around() {
        let counter = Arc::new(Mutex::new(98u32));
        let miss = Arc::new(Mutex::new(0u32));

        // Simuler 3 incréments
        for _ in 0..3 {
            let mut c = counter.lock().unwrap();
            *c += 1;
            if *c >= 100 {
                *c = 0;
                let mut m = miss.lock().unwrap();
                *m += 1;
            }
        }

        assert_eq!(*counter.lock().unwrap(), 1);
        assert_eq!(*miss.lock().unwrap(), 1);
    }
}
