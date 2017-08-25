#![feature(inclusive_range_syntax)]

extern crate rand;

mod sim;

fn main()
{
  let mut game =
    sim::LifeSimulator::new(sim::LifeRules::PercentageRules, 2, 100);

  /*
   *game.mut_coords(&[50, 50]) = sim::Cell::Alive;
   *game.mut_coords(&[51, 50]) = sim::Cell::Alive;
   *game.mut_coords(&[51, 48]) = sim::Cell::Alive;
   *game.mut_coords(&[53, 49]) = sim::Cell::Alive;
   *game.mut_coords(&[54, 50]) = sim::Cell::Alive;
   *game.mut_coords(&[55, 50]) = sim::Cell::Alive;
   *game.mut_coords(&[56, 50]) = sim::Cell::Alive;
   */
  game.randomize_grid();

  for y in 0..100
  {
    for x in 0..100
    {
      print!("{}", game.get_cell(game.coords_to_index(&[x, y])));
    }
    println!("");
  }

  loop
  {
    game.step();

    println!("");
    for y in 0..100
    {
      for x in 0..100
      {
        print!("{}", game.get_cell(game.coords_to_index(&[x, y])));
      }
      println!("");
    }
  }
}
