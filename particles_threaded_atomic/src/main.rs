use rand::random;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use std::thread;

// Constants
const NUM_OF_PARTICLES: usize = 100;
const ENCLOSURE_SIZE: f32 = 10.0; // 10x10 enclosure
const MOVE_DURATION: u64 = 10; // Move particles for 10 seconds
const COLLISION_THRESHOLD: f32 = 0.2; // Threshold for considering a collision

// Define the Particle struct
#[derive(Debug, Copy, Clone)]
struct Particle {
    x: f32,
    y: f32,
}

impl Particle {
    // Create a new particle with random initial position within the enclosure
    fn new() -> Self {
        let x = random::<f32>() * ENCLOSURE_SIZE;
        let y = random::<f32>() * ENCLOSURE_SIZE;
        Particle { x, y }
    }

    // Move the particle by a random distance within the enclosure
    fn move_particle(&mut self) {
        let dx = (random::<f32>() - 0.5) * 2.0; // Random value between -1 and 1
        let dy = (random::<f32>() - 0.5) * 2.0; // Random value between -1 and 1

        self.x = (self.x + dx).clamp(0.0, ENCLOSURE_SIZE);
        self.y = (self.y + dy).clamp(0.0, ENCLOSURE_SIZE);
    }

    // Check if this particle collides with another
    fn collide(&self, other: &Particle) -> bool {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let distance = (dx * dx + dy * dy).sqrt();
        distance < COLLISION_THRESHOLD
    }

    // Get the position of the particle
    fn get_position(&self) -> (f32, f32) {
        (self.x, self.y)
    }
}

// Define the ParticleSystem struct
struct ParticleSystem {
    particles: Vec<Particle>,
    collision_count: Arc<AtomicUsize>, // Atomic counter for collisions
}

impl ParticleSystem {
    // Create a new ParticleSystem with a specified number of particles
    fn new() -> Self {
        let particles = (0..NUM_OF_PARTICLES)
            .map(|_| Particle::new())
            .collect::<Vec<Particle>>();
        let collision_count = Arc::new(AtomicUsize::new(0)); // Initialize the atomic counter
        ParticleSystem { particles, collision_count }
    }

    // Move all particles within the system
    fn move_particles(&mut self) {
        for particle in &mut self.particles {
            particle.move_particle();
        }
    }

    // Get the number of particles
    fn get_particle_count(&self) -> usize {
        self.particles.len()
    }

    // Get all particle positions for testing
    fn get_particle_positions(&self) -> Vec<(f32, f32)> {
        self.particles.iter().map(|p| p.get_position()).collect()
    }

    fn check_collisions(&self) -> usize {
        let mut collision_count = 0;
        for i in 0..self.particles.len() {
            for j in (i + 1)..self.particles.len() {
                if self.particles[i].collide(&self.particles[j]) {
                    collision_count += 1;
                }
            }
        }
        collision_count
    }

    // Get the total number of collisions
    fn get_collision_count(&self) -> usize {
        self.collision_count.load(Ordering::SeqCst)
    }
}

fn main() {
    // Initialize the particle system
    let system = Arc::new(Mutex::new(ParticleSystem::new()));

    // Print initial positions
    //let system_clone = Arc::clone(&system);
    //let initial_positions = {
    //    let system = system_clone.lock().unwrap();
    //    system.get_particle_positions()
    //};

    //println!("Initial positions:");
    //for (i, pos) in initial_positions.iter().enumerate() {
    //    println!("Particle {}: ({}, {})", i, pos.0, pos.1);
    //}

    //thread_main

    // Create threads to move particles
    let move_thread = {
        let system = Arc::clone(&system);
        thread::spawn(move || {
            let start_time = Instant::now();
            while start_time.elapsed() < Duration::new(MOVE_DURATION, 0) {
                // Move particles with exclusive lock
                let mut system = system.lock().unwrap();
                system.move_particles();
            }
        })
    };

    // Create threads to check for collisions
    let collision_thread = {
        let system = Arc::clone(&system);
        let collision_count = Arc::clone(&system.lock().unwrap().collision_count);
        thread::spawn(move || {
            let start_time = Instant::now();
            while start_time.elapsed() < Duration::new(MOVE_DURATION, 0) {
                // Check for collisions with the particles
                let system = system.lock().unwrap();
                let collisions = system.check_collisions();
                collision_count.fetch_add(collisions, Ordering::SeqCst);
            }
        })
    };

    // Wait for threads to finish
    move_thread.join().unwrap();
    collision_thread.join().unwrap();


    // Print collision count
    let system_clone1 = Arc::clone(&system); // Clone the Arc before locking
    //let system_clone2 = Arc::clone(&system);
    let system = system_clone1.lock().unwrap();
    println!("\nTotal collisions: {}", system.get_collision_count());
    //println!("\nUpdated positions after simulation:");
    //for (i, pos) in system.get_particle_positions().iter().enumerate() {
    //println!("Particle {}: ({}, {})", i, pos.0, pos.1);
    //}

}
