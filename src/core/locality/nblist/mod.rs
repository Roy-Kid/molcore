#[cfg(feature = "rayon")]
pub mod rayon {
    //! Re-export of the [`ParallelIterator`] trait.
    pub use rayon::prelude::ParallelIterator;
}

pub mod linked_cell;

#[allow(dead_code)]
#[cfg(test)]
mod tests {



}
