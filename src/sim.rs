use rand::{self, Rng};

/// A single cell in the simulation
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell
{
  Dead,
  Alive,
}

impl ::std::convert::From<bool> for Cell
{
  fn from(val: bool) -> Cell
  {
    if val { Cell::Alive } else { Cell::Dead }
  }
}

impl ::std::fmt::Display for Cell
{
  fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
  {
    // This exists mainly for debugging
    write!(f, "{}", if *self == Cell::Alive { "█" } else { "░" })
  }
}

/// There are multiple sets of rules for a simulation that work differently.
/// Given the number of dimensions d and the number of neighbors n (3^d - 1),
///
/// * `BasicRules`
/// ..* A cell will die if there are less than floor(n / 4) alive neighbors.
/// ..* A cell will die if there are more than floor((n + 1) / 3) alive neighbors.
/// ..* A cell will come to life if there are exactly floor((n + 1) / 3) alive neighbors.
/// * `PercentageRules`
/// ..* A cell will die if there are less than 0.25n alive neighbors.
/// ..* A cell will die if there are more than 0.40625n alive neighbors.
/// ..* A cell will come to life if the number of alive neighbors a is
///     0.34375n <= a <= 0.40625n.
pub enum LifeRules
{
  BasicRules,
  PercentageRules,
}

/// A simulation of Conway's Game of Life, generalized to N-dimensions.
#[derive(Clone)]
pub struct LifeSimulator
{
  cells: Vec<Cell>,
  dim: u32,
  size: usize, // number of cells in each dimension
  min_neighbors: u32, // min number of neighbors to live
  min_breed_neighbors: u32, // min number of neighbors needed to come to life
  max_neighbors: u32, // max number of neighbors before death
}

#[allow(dead_code)]
impl LifeSimulator
{
  /// Creates a new simulation with the given rules and number of dimensions.
  /// The `size` parameter is the size in all dimensions, that is a 4D simulation
  /// will have a `size` x `size` x `size` x `size` grid.
  /// All cells will be initialized as `sim::Cell::Dead`.
  pub fn new(rules: LifeRules, dimensions: u32, size: usize) -> Self
  {
    let num_neighbors = 3u32.pow(dimensions) - 1;
    let min_neighbors = match rules
    {
      LifeRules::BasicRules => num_neighbors / 4,
      LifeRules::PercentageRules => num_neighbors / 4,
    };
    let min_breed_neighbors = match rules
    {
      LifeRules::BasicRules => (num_neighbors + 1) / 3,
      LifeRules::PercentageRules =>
      {
        ((num_neighbors as f64) * 0.34375).ceil() as u32
      }
    };
    let max_neighbors = match rules
    {
      LifeRules::BasicRules => (num_neighbors + 1) / 3,
      LifeRules::PercentageRules => ((num_neighbors as f64) * 0.40625) as u32,
    };

    LifeSimulator {
      cells: vec![Cell::Dead; size.pow(dimensions)],
      dim: dimensions,
      size: size,
      min_neighbors: min_neighbors,
      min_breed_neighbors: min_breed_neighbors,
      max_neighbors: max_neighbors,
    }
  }

  /// Randomizes the state of all cells in the simulation.
  pub fn randomize_grid(&mut self)
  {
    let mut rng = rand::thread_rng();
    for cell in self.cells.iter_mut()
    {
      *cell = Cell::from(rng.gen::<bool>());
    }
  }

  fn tagged_coords_to_index(size: usize, dim: u32, coords: &[u32]) -> usize
  {
    // For internal use only
    assert_eq!(coords.len(), dim as usize);

    let mut index = 0;

    for (d, coord) in (1...dim).zip(coords.iter())
    {
      index += size.pow(d - 1) * *coord as usize;
    }

    index
  }
  fn tagged_index_to_coords(size: usize, dim: u32, index: usize) -> Vec<u32>
  {
    // For internal use only
    let mut index = index;
    let mut tmp_coord;
    let mut tmp_coord_scale;
    let mut coords = Vec::with_capacity(dim as usize);

    for d in (0...(dim - 1)).rev()
    {
      tmp_coord_scale = size.pow(d);
      tmp_coord = index / tmp_coord_scale;

      index -= tmp_coord * tmp_coord_scale;
      coords.insert(0, tmp_coord as u32);
    }

    coords
  }

  /// Creates an index given a set of coordinates.
  /// The number of coordinates must be the same as the number of dimensions for
  /// this simulation.
  /// The index may be used to retrieve a cell.
  pub fn coords_to_index(&self, coords: &[u32]) -> usize
  {
    LifeSimulator::tagged_coords_to_index(self.size, self.dim, coords)
  }
  /// Creates a set of coordinates given an index.
  pub fn index_to_coords(&self, index: usize) -> Vec<u32>
  {
    LifeSimulator::tagged_index_to_coords(self.size, self.dim, index)
  }

  /// Gets the cell at the given `index`.
  pub fn get_cell(&self, index: usize) -> Cell
  {
    self.cells[index]
  }

  /// Returns a mutable reference to the cell at the given `index`.
  pub fn mut_cell(&mut self, index: usize) -> &mut Cell
  {
    &mut self.cells[index]
  }

  /// Get the indices for the neighbors of the cell at the given `index`.
  pub fn get_neighbor_indices(&self, index: usize) -> Vec<usize>
  {
    // Neighborhood is # of neighbors + 1 (+1 to include the cell itself)
    let neighborhood = 3usize.pow(self.dim);
    let mut n_inds = Vec::new();
    // Coordinates equivalent to the index
    let coords = self.index_to_coords(index);

    'neighbor: for n in 1..neighborhood
    {
      let mut n = n;
      // This will hold the coordinates of a neighbor
      let mut n_coords = coords.clone();

      for d in (0...(self.dim as usize - 1)).rev()
      {
        let tern_digit = 3usize.pow(d as u32); // The digit in base-3
        // n_d represents where the neighbor is relative to the cell in the
        // dimension d.
        // 0 means the neighbor has the same coordinate for the dimension d.
        // 1 means the neighbor comes before the cell in dimension d.
        // 2 means the neighbor comes after the cell in dimension d.
        let n_d = n / tern_digit;
        // Doing this subtraction allows n_d to be calculated in lower
        // dimensions easily.
        n -= n_d * tern_digit;

        n_coords[d] = match n_d
        {
          1 =>
          {
            match n_coords[d].checked_sub(1)
            {
              Option::Some(val) => val,
              Option::None => break 'neighbor, // No neighbor will exist < 0
            }
          }
          2 =>
          {
            match n_coords[d].checked_add(1)
            {
              Option::Some(val) => val,
              Option::None => break 'neighbor,
            }
          }
          _ => n_coords[d],
        };
        // The neighbor only truly exists if it is within bounds.
        // The underflow check before checks for minimum bounds, and below
        // checks for maximum bounds.
        if n_coords[d] as usize == self.size
        {
          break 'neighbor;
        }
      }

      // Convert the coordinates to an index and add it to the list
      n_inds.push(self.coords_to_index(&n_coords));
    }

    n_inds
  }

  /// Performs a single step in the simulation.
  pub fn step(&mut self)
  {
    let last_state = self.clone();
    let mut alive_neighbors;

    for (index, cell) in self.cells.iter_mut().enumerate()
    {
      // Get number of alive neighbors
      alive_neighbors = 0;
      let n_inds = last_state.get_neighbor_indices(index);
      for n_ind in n_inds
      {
        if last_state.get_cell(n_ind) == Cell::Alive
        {
          alive_neighbors += 1;
        }
      }

      // Check whether the cell should be set to dead, set to alive, or kept
      // in its current state
      if alive_neighbors < last_state.min_neighbors ||
        alive_neighbors > last_state.max_neighbors
      {
        *cell = Cell::Dead;
      }
      else if alive_neighbors >= last_state.min_breed_neighbors &&
               alive_neighbors <= last_state.max_neighbors
      {
        *cell = Cell::Alive;
      }
    }
  }
}
