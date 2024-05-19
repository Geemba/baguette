use std::{marker::PhantomData, ops::{Deref, DerefMut}, ptr::NonNull};

use crate::*;

use self::sprite::SpriteImpl;

pub struct SpriteSheet
{
    pub inner: Sprite,
    sections: Vec<SliceSection>,
}

impl Deref for SpriteSheet
{
    type Target = Sprite;

    fn deref(&self) -> &Self::Target
    {
        &self.inner
    }
}

impl DerefMut for SpriteSheet
{ 
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.inner
    }
}

impl SpriteSheet
{
    pub fn new(renderer: &mut crate::Renderer, loader: SpriteSheetLoader) -> Self
    {
        Self
        {
            inner: Sprite::new(renderer, loader.inner),
            sections: loader.sections,
        }
    }

    pub fn iter(&mut self) -> Iter
    {
        Iter
        {
            sections: &self.sections,
            idx: 0,
            sprite: &self.inner.sprite,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut
    {
        IterMut
        {
            sections: (&mut self.sections).into(),
            idx: 0,
            sprite: (&mut *self.inner.sprite).into(),
            _phantom: PhantomData,
        }
    }
}

pub struct Iter<'a>
{
    sections: &'a Vec<SliceSection>,
    sprite: &'a SpriteImpl,
    idx: usize
}

impl<'a> Iterator for Iter<'a>
{
    type Item = (&'a SpriteInstance, &'a SliceSection);

    fn next(&mut self) -> Option<Self::Item>
    {
        let item = match self.sprite.instances.get(self.idx)
        {
            Some(instance) => Some((instance, &self.sections[self.idx])),
            None => None,
        };

        self.idx += 1;
        item
    }
}

pub struct IterMut<'a>
{
    sections: NonNull<Vec<SliceSection>>,
    sprite: NonNull<SpriteImpl>,
    idx: usize,
    _phantom: PhantomData<&'a()>
}

impl<'a> Drop for IterMut<'a>
{
    fn drop(&mut self)
    {
        let instances = unsafe { &mut self.sprite.as_mut().instances };
        let sections = unsafe { self.sections.as_mut() };

        for (i, instance) in instances.iter_mut().enumerate()
        {
            instance.uv_idx = sections[i].index
        }
    }
}

impl<'a> Iterator for IterMut<'a>
{
    type Item = (&'a mut SpriteInstance, &'a mut SliceSection);

    fn next(&mut self) -> Option<Self::Item>
    {
        let instances = unsafe { &mut self.sprite.as_mut().instances };
        let sections = unsafe { self.sections.as_mut() };

        let item = match instances.get_mut(self.idx)
        {
            Some(instance) => Some((instance, &mut sections[self.idx])),
            None => None,
        };
        
        self.idx += 1;
        item
    }
}

/// describes the type of sprite you want to create
pub struct SpriteSheetLoader
{
    inner: SpriteLoader,
    sections: Vec<SliceSection>,
}

impl SpriteSheetLoader
{
    pub fn new<'a>
    (
        path: impl Into<std::ffi::OsString>,
        instances: impl IntoIterator<Item = (SpriteInstance, SheetSlices<'a>)>,
        rows: u32,
        columns: u32
    )
    -> Self
    {
        let mut instances = instances.into_iter().collect::<Vec<_>>();

        if instances.is_empty()
        {
            instances.push((Default::default(), SheetSlices::All));
        }

        let inner = SpriteLoader::new(path)
            .instances(instances.iter().map(|(instance, ..)| instance.clone()))
            .slice_atlas(rows, columns);       

        let SpriteLoader { rows, columns, .. } = inner;

        let sections = instances.iter().map(|(..,tiles)|
        {
            SliceSection
            {
                rows,
                columns,
                indices: tiles.clone().into_indices(rows, columns),
                index: 0
            }
        }).collect();

        Self
        {
            inner,
            sections
        }
    }

    pub fn new_pixelated<'a>
    (
        path: impl Into<std::ffi::OsString>,
        instances: impl IntoIterator<Item = (SpriteInstance, SheetSlices<'a>)>,
        rows: u32,
        columns: u32
    )
    -> Self
    {
        let mut this = Self::new(path, instances, rows, columns);
        this.inner.filtermode = FilterMode::Nearest;  
        this
    }
}

#[derive(Clone)]
pub struct Tiles { index: usize, indices: Box<[u32]> }

#[derive(Clone, Default)]
pub struct SliceSection
{
    rows: u32,
    columns: u32,

    indices: Option<Tiles>,
    /// the index into the uv buffer for this section
    index: u32
}

impl SliceSection
{
    pub fn empty() -> Self
    {
        Self
        {
            rows: 1,
            columns: 1,

            indices: None,
            index: 0,
        }
    }

    /// sets the index of the sheet to render with the provided row and column value
    pub fn set(&mut self, row: u32, column: u32)
    {
        // we clamp the max value possible to the length of the uv buffer, whose value is
        // determined by (rows * columns -1 )
        self.index = u32::min
        (
            column * self.rows + row,
            (self.rows * self.columns) -1
        )
    }

    /// sets the index of the spritesheet's section to the next one
    /// or the first one if it exceeds the maximum avaiable index
    pub fn next_or_first(&mut self)
    {
        self.index = match self.indices
        {
            Some(ref mut tiles) => match tiles.indices.get(tiles.index + 1)
            {
                Some(&next_index) =>
                {
                    tiles.index += 1;
                    next_index
                }
                None =>
                {
                    tiles.index = 0;
                    tiles.indices[0]
                }
            }
            
            None => match self.index <= (self.rows * self.columns - 1)
            {
                true => self.index + 1,
                false => 0
            }
        }
    }

    /// set which indices will be avaiable for playing.
    /// 
    /// # examples
    /// 
    /// accepted values are:
    /// 
    /// * [std::ops::Range]
    /// ```
    ///     section.set_indices(0..2);
    /// ```
    /// * [std::ops::RangeInclusive]
    /// ```
    ///     [std::ops::Range]
    ///     section.set_indices(0..=1);
    /// ```
    /// 
    /// * iterators of type [(u32, u32)] where the first integer represents the `row`
    /// 
    ///     while the second represents the `comumn`:
    /// 
    /// ```
    ///     section.set_indices([(0,0),(1,0),(2,0),(3,0)]);
    /// 
    ///     section.set_indices(vec![(0,1),(1,1),(2,1)]);
    /// ```
    pub fn set_indices(&mut self, items: impl IntoIndices)
    {
        self.indices = items.into_indices(self.rows, self.columns);
        self.index = match &self.indices
        {
            Some(tiles) => tiles.index as u32,
            None => 0
        }
    }
}

pub trait IntoIndices 
{
    /// converts an iteration to an array of indices
    /// aligned to the correct uv instance uvs
    fn into_indices(self, rows: u32, columns: u32) -> Option<Tiles> ;
}

impl IntoIndices for std::ops::Range<u32>
{
    fn into_indices(self, rows: u32, columns: u32) -> Option<Tiles>
    {
        let indices = self.into_iter()
            .filter(|i| *i < rows*columns)
            .collect::<Box<[u32]>>();

        match indices.len()>0 
        {
            true => Some(Tiles { index:0, indices }),
            false => None 
        }
    }
}

impl IntoIndices for std::ops::RangeInclusive<u32>
{
    fn into_indices(self, rows: u32, columns: u32) -> Option<Tiles>
    {
        let indices = self.into_iter()
            .filter(|i| *i < rows*columns)
            .collect::<Box<[u32]>>();

        match indices.len()>0 
        {
            true => Some(Tiles { index:0, indices }),
            false => None 
        }
    }
}

impl IntoIndices for std::ops::RangeFull
{
    fn into_indices(self, _: u32, _: u32) -> Option<Tiles>
    {
        None 
    }
}

impl IntoIndices for ()
{
    fn into_indices(self,  _: u32, _: u32) -> Option<Tiles>
    {
        None 
    }
}

impl IntoIndices for &[u32]
{
    fn into_indices(self, rows: u32, columns: u32) -> Option<Tiles>
    {
        let indices = self.iter()
            .copied()
            .filter(|i| *i < rows*columns-1)
            .collect::<Box<[u32]>>();

        match indices.len()>0 
        {
            true => Some(Tiles { index:0, indices }),
            false => None 
        }
    }
}

impl IntoIndices for &[(u32,u32)]
{
    fn into_indices(self,  rows: u32, columns: u32) -> Option<Tiles>
    {
        let indices = self.iter()
            .map(|(row,column)| column * rows + row)
            .filter(|i| *i < rows*columns-1)
            .collect::<Box<[u32]>>();

        match indices.len() > 0 
        {
            true => Some(Tiles {index:0,indices}),
            false => None
        }
    }
}

#[derive(Clone)]
pub enum SheetSlices<'a>
{
    /// like most indexing operations, the count starts from zero, so `0`
    /// returns the first tile, `1` the second, and so on
    Set(&'a [u32]),
    /// like most indexing operations, the count starts from zero, so `0`
    /// returns the first tile, `1` the second, and so on
    RowColumn(&'a[(u32,u32)]),
    /// specify the tile indices using a [`std::ops::Range`] (`start..end`)
    /// 
    /// like most indexing operations, the count starts from zero, so `0`
    /// returns the first tile, `1` the second, and so on
    Range(std::ops::Range<u32>),
    /// specify the tile indices using a [`std::ops::RangeInclusive`] (`start..=end`)
    /// 
    /// like most indexing operations, the count starts from zero, so `0`
    /// returns the first tile, `1` the second, and so on
    RangeIn(std::ops::RangeInclusive<u32>),
    All
}

impl SheetSlices<'_>
{
    pub(crate) fn into_indices(self, rows: u32, columns: u32) -> Option<spritesheet::Tiles>
    {
        use spritesheet::IntoIndices;
        
        match self
        {
            SheetSlices::Set(val) => val.into_indices(rows, columns),
            SheetSlices::RowColumn(val) => val.into_indices(rows, columns),
            SheetSlices::Range(val) => val.into_indices(rows, columns),
            SheetSlices::RangeIn(val) => val.into_indices(rows, columns),
            SheetSlices::All => None,
        }
    }
}