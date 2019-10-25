#[repr(C)]
pub struct Collection<T> {
    data: *mut T,
    count: u16,
    size: u16
}

impl<T> Collection<T> {
    pub fn begin(&self) -> *mut T {
        self.data
    }

    pub fn end(&self) -> *mut T {
        unsafe { self.data.add(self.count as usize) }
    }

    pub fn at(&self, index: u16) -> *mut T {
        unsafe { self.data.add(index as usize) }
    }

    pub fn iter(&self) -> CollectionIterator<T> {
        CollectionIterator {
            collection: self,
            index: 0
        }
    }
}

#[repr(C)]
pub struct PtrCollection<T> {
    data: *mut *mut T,
    count: u16,
    size: u16
}

impl<T> PtrCollection<T> {
    pub fn begin(&self) -> *mut *mut T {
        self.data
    }

    pub fn end(&self) -> *mut *mut T {
        unsafe { self.data.add(self.count as usize) }
    }

    pub fn at(&self, index: u16) -> *mut T {
        unsafe { self.data.add(index as usize).read() }
    }

    pub fn count(&self) -> u16 {
        self.count
    }

    pub fn set(&mut self, index: u16, ptr: *mut T) {
        unsafe { self.data.add(index as usize).write(ptr) }
    }

    pub fn iter(&self) -> PtrCollectionIterator<T> {
        PtrCollectionIterator {
            collection: self,
            index: 0
        }
    }
}

pub struct PtrCollectionIterator<'a, T> {
    collection: &'a PtrCollection<T>,
    index: u16
}

impl<'a, T> Iterator for PtrCollectionIterator<'a, T> {
    type Item = *mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.collection.count {
            let index = self.index;
            self.index += 1;
            Some(self.collection.at(index))
        } else {
            None
        }
    }
}

pub struct CollectionIterator<'a, T> {
    collection: &'a Collection<T>,
    index: u16
}

impl<'a, T> Iterator for CollectionIterator<'a, T> {
    type Item = *mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.collection.count {
            let index = self.index;
            self.index += 1;
            Some(self.collection.at(index))
        } else {
            None
        }
    }
}