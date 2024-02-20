#[derive(Debug, Clone)]
pub struct DenseMatrix<T> {
    data: Box<[T]>,
    shape: Shape,
}

impl<T: Default> DenseMatrix<T> {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self::new_with(rows, cols, || T::default())
    }
}

impl<T> DenseMatrix<T> {
    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    pub fn row_view(&self, i: usize) -> &[T] {
        &self.data[self.shape.row_view_range(i)]
    }

    pub fn row_view_mut(&mut self, i: usize) -> &mut [T] {
        &mut self.data[self.shape.row_view_range(i)]
    }

    pub fn iter_rows(&self) -> IterRows<'_, T> {
        IterRows {
            data: self,
            cursor: 0,
        }
    }

    pub fn new_with<F>(rows: usize, cols: usize, f: F) -> Self
    where
        F: FnMut() -> T,
    {
        let shape = Shape::new(rows, cols);
        let mut data = Vec::with_capacity(shape.total());
        data.resize_with(shape.total(), f);
        Self {
            data: data.into(),
            shape,
        }
    }
}

#[derive(Debug, Clone)]
struct Shape {
    rows: usize,
    cols: usize,
}

impl Shape {
    fn new(rows: usize, cols: usize) -> Self {
        Self { rows, cols }
    }

    #[inline(always)]
    fn total(&self) -> usize {
        self.rows * self.cols
    }

    #[inline(always)]
    fn row_view_range(&self, i: usize) -> std::ops::Range<usize> {
        let start = i * self.cols;
        start..start + self.cols
    }
}

pub struct IterRows<'a, T> {
    data: &'a DenseMatrix<T>,
    cursor: usize,
}

impl<'a, T> Iterator for IterRows<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor < self.data.shape.rows {
            let cursor = self.cursor;
            self.cursor += 1;
            Some(self.data.row_view(cursor))
        } else {
            None
        }
    }
}
