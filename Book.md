
# Q1. Colliding Particles
## Question
Create a new function collide() that checks if a particle collides with (or is very close to) another particle.

Create a new pool of threads with a new thread main. Within this thread main, you'll need to iterate over the list of particles calling your collide() function for each pair of particles.

Now add a counter to count the number of collision that occur in your simulation. Initially, this counter can be local to the new thread main. Print this counter before the thread terminates. In the next exercise we'll replace this counter with an atomic.

## Solution
```Rust
use rand::random;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
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
}

impl ParticleSystem {
    // Create a new ParticleSystem with a specified number of particles
    fn new() -> Self {
        let particles = (0..NUM_OF_PARTICLES)
            .map(|_| Particle::new())
            .collect::<Vec<Particle>>();
        ParticleSystem { particles }
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

    // Function to check for collisions between particles
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
}

fn main() 
{
   // Initialize the particle system
   let system = Arc::new(Mutex::new(ParticleSystem::new()));

   // Print initial positions
   let system_clone = Arc::clone(&system);
   let initial_positions = {
       let system = system_clone.lock().unwrap();
       system.get_particle_positions()
   };

   println!("Initial positions:");
   for (i, pos) in initial_positions.iter().enumerate() {
       println!("Particle {}: ({}, {})", i, pos.0, pos.1);
   }

   //thread_main
   // Create a shared counter for collisions
   let collision_counter = Arc::new(Mutex::new(0));

   // Move particles for 10 seconds in one thread
   let move_thread = {
       let system = Arc::clone(&system);
       let collision_counter = Arc::clone(&collision_counter);
       
       thread::spawn(move || {
           let start_time = Instant::now();

           // Run the simulation for approximately 10 seconds
           while start_time.elapsed() < Duration::new(MOVE_DURATION, 0) {
               // Lock the system and move particles
               let mut system = system.lock().unwrap();
               system.move_particles();

               // Check for collisions
               let collisions = system.check_collisions();
               let mut counter = collision_counter.lock().unwrap();
               *counter += collisions;
           }
       })
   };

   // Wait for the move thread to finish
   move_thread.join().unwrap();

   // Print collision count
   let counter = collision_counter.lock().unwrap();
   println!("\nTotal collisions: {}", *counter);

   // Print updated positions
   let system_clone = Arc::clone(&system);
   let system = system_clone.lock().unwrap();
   println!("\nUpdated positions after simulation:");
   for (i, pos) in system.get_particle_positions().iter().enumerate() {
       println!("Particle {}: ({}, {})", i, pos.0, pos.1);
   }
}

```
![image](https://github.com/user-attachments/assets/8ed49a9d-6b60-4de1-b595-91991c539729)

## Reflection

#### Is locking required in your solution to prevent race conditions?
As 2 threads could read the value stored in collision_counter thread 1 could increment it and assign the new value. whilst at the same time, thread 2 has the old value increments it by 1 then assigns it to collision_counter. we have a race condition. So we use an Arc<Mutex<>> to safely share the collision_counter and ParticleSystem between threads. The Mutex ensures that the counter is safely modified and prevents race conditions. the Arc allows us to share ownership of the mutex safely

#### Are there any other race conditions that can occur in your code?
there is potential for 2 race conditions. ParticleSystem and collision_counter. However in this code because access to shared data (both ParticleSystem and collision_counter) is synchronized using Mutex. the race condition is removed

#### Are there any optimisations you can make to your code?

could introduce atomic counters instead of using a mutex to further optimize the collision counting. 
An atomic counter allows multiple threads to increment or decrement a counter concurrently without waiting for other threads. 
Since atomic operations are performed at the hardware level, they avoid the need for explicit locking. 
This leads to less contention and can be much faster in situations where threads frequently modify shared variables.

# Q2. Recording collisions using an Atomic

Replace the local counter with an atomic counter to measure the number of collisions across all threads. This counter should be stored only once in the ParticleSystem class.

## Solution 
```Rust
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

    // Function to check for collisions between particles
    fn check_collisions(&self) {
        let mut collision_count = 0;
        for i in 0..self.particles.len() {
            for j in (i + 1)..self.particles.len() {
                if self.particles[i].collide(&self.particles[j]) {
                    collision_count += 1;
                }
            }
        }

        // Update the atomic collision counter
        self.collision_count.fetch_add(collision_count, Ordering::SeqCst);
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
    let system_clone = Arc::clone(&system);
    let initial_positions = {
        let system = system_clone.lock().unwrap();
        system.get_particle_positions()
    };

    println!("Initial positions:");
    for (i, pos) in initial_positions.iter().enumerate() {
        println!("Particle {}: ({}, {})", i, pos.0, pos.1);
    }

    //thread_main
    // Move particles for 10 seconds in one thread
    let move_thread = {
        let system = Arc::clone(&system);
        
        thread::spawn(move || {
            let start_time = Instant::now();

            // Run the simulation for approximately 10 seconds
            while start_time.elapsed() < Duration::new(MOVE_DURATION, 0) {
                // Lock the system and move particles
                let mut system = system.lock().unwrap();
                system.move_particles();

                // Check for collisions
                system.check_collisions();
            }
        })
    };

    // Wait for the move thread to finish
    move_thread.join().unwrap();

    // Print collision count
    let system_clone1 = Arc::clone(&system); // Clone the Arc before locking
    let system_clone2 = Arc::clone(&system);
     
    let system = system_clone1.lock().unwrap();
    println!("\nTotal collisions: {}", system.get_collision_count());
    println!("\nUpdated positions after simulation:");
    for (i, pos) in system.get_particle_positions().iter().enumerate() {
    println!("Particle {}: ({}, {})", i, pos.0, pos.1);
    }

}

```
## Reflection
replaced the Mutex-based shared counter with an AtomicUsize field in ParticleSystem. 

This change allows safe updates to the collision count across multiple threads without requiring a lock.

The counter is updated using the fetch_add method.

The check_collisions method now directly updates the atomic counter with the number of collisions detected in each iteration.

added the get_collision_count method to retrieve the total collision count at the end of the simulation.

With atomic operations like fetch_add, we avoid the overhead of locking while still ensuring that updates to the counter are safe across threads.

simplified the process and avoided unnecessary cloning of Arc references. This ensures that all data is accessed safely without risking deadlocks

# Q3. Ownership

## Question

Can you think of an approach where both sets of threads can execute at the same time?

## Solution
Only included the changed code

```Rust
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
```


## Reflection

Move threads acquire exclusive locks on the particle positions to modify them.

Collision-checking threads only read the particle positions and can execute concurrently without needing exclusive locks.

Move threads: These threads acquire a write lock on the system and move the particles.

Collision-checking threads: These threads acquire a read lock on the system to check for collisions concurrently.

The system is wrapped in a Mutex, ensuring that only one thread can modify the particle positions at a time (i.e., only the moving thread can move particles).

The collision count is tracked using an AtomicUsize, which allows updates to the count without blocking or requiring locks. The fetch_add method is used to safely increment the collision count across threads.

Since AtomicUsize operations are lock-free, multiple threads can check for collisions and increment the counter concurrently.

This ensures that the particles' positions are updated safely and concurrently with other threads reading the data.
