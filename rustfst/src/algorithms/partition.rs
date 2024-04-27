use crate::algorithms::Queue;
use crate::StateId;

#[derive(Debug, Clone)]
pub struct Partition {
    elements: Vec<Element>,
    classes: Vec<Class>,
    visited_classes: Vec<usize>,
    yes_counter: usize,
}

impl Partition {
    pub fn empty_new() -> Self {
        Self {
            elements: vec![],
            classes: vec![],
            visited_classes: vec![],
            yes_counter: 0,
        }
    }

    pub fn new(num_elements: usize) -> Self {
        let mut c = Self::empty_new();
        c.initialize(num_elements);
        c
    }

    pub fn initialize(&mut self, num_elements: usize) {
        self.elements.resize_with(num_elements, Element::new);
        self.classes.clear();
        self.classes.reserve(num_elements);
        self.yes_counter = 1;
    }

    pub fn add_class(&mut self) -> usize {
        let num_class = self.classes.len();
        self.classes.resize_with(num_class + 1, Class::new);
        num_class
    }

    pub fn allocate_classes(&mut self, num_additional_classes: usize) {
        let num_class = self.classes.len();
        self.classes
            .resize_with(num_class + num_additional_classes, Class::new);
    }

    pub fn add(&mut self, element_id: usize, class_id: usize) {
        let this_class = &mut self.classes[class_id];
        this_class.size += 1;

        let no_head = this_class.no_head;
        if no_head >= 0 {
            self.elements[no_head as usize].prev_element = element_id as i32;
        }
        this_class.no_head = element_id as i32;

        let this_element = &mut self.elements[element_id];
        this_element.class_id = class_id;
        this_element.yes = 0;
        this_element.next_element = no_head;
        this_element.prev_element = -1;
    }

    pub fn move_element(&mut self, element_id: usize, class_id: usize) {
        let elt_prev_elt = self.elements[element_id].prev_element;
        let elt_next_elt = self.elements[element_id].next_element;
        let elt_class_id = self.elements[element_id].class_id;

        let old_class = &mut self.classes[elt_class_id];
        old_class.size -= 1;

        if elt_prev_elt >= 0 {
            self.elements[elt_prev_elt as usize].next_element = elt_next_elt;
        } else {
            old_class.no_head = elt_next_elt;
        }

        if elt_next_elt >= 0 {
            self.elements[elt_next_elt as usize].prev_element = elt_prev_elt;
        }

        self.add(element_id, class_id)
    }

    pub fn split_on(&mut self, element_id: usize) {
        let elt_yes = self.elements[element_id].yes;
        let elt_class_id = self.elements[element_id].class_id;
        let elt_prev_elt = self.elements[element_id].prev_element;
        let elt_next_elt = self.elements[element_id].next_element;

        let this_class = &mut self.classes[elt_class_id];

        if elt_yes == self.yes_counter {
            return;
        }

        if elt_prev_elt >= 0 {
            self.elements[elt_prev_elt as usize].next_element = elt_next_elt;
        } else {
            this_class.no_head = elt_next_elt;
        }

        if elt_next_elt >= 0 {
            self.elements[elt_next_elt as usize].prev_element = elt_prev_elt;
        }

        if this_class.yes_head >= 0 {
            self.elements[this_class.yes_head as usize].prev_element = element_id as i32;
        } else {
            self.visited_classes.push(elt_class_id);
        }

        self.elements[element_id].yes = self.yes_counter;
        self.elements[element_id].next_element = this_class.yes_head;
        self.elements[element_id].prev_element = -1;
        this_class.yes_head = element_id as i32;
        this_class.yes_size += 1;
    }

    pub fn split_refine(&mut self, class_id: usize) -> i32 {
        let yes_size = self.classes[class_id].yes_size;
        let size = self.classes[class_id].size;
        let no_size = size - yes_size;
        if no_size == 0 {
            // All members are in the 'yes' subset, so we don't have to create a new
            // class, just move them all to the 'no' subset.
            self.classes[class_id].no_head = self.classes[class_id].yes_head;
            self.classes[class_id].yes_head = -1;
            self.classes[class_id].yes_size = 0;
            -1
        } else {
            let new_class_id = self.classes.len();
            self.classes.resize_with(self.classes.len() + 1, Class::new);
            // The new_class will have the values from the constructor.
            if no_size < yes_size {
                // Moves the 'no' subset to new class ('no' subset).
                self.classes[new_class_id].no_head = self.classes[class_id].no_head;
                self.classes[new_class_id].size = no_size;
                // And makes the 'yes' subset of the old class ('no' subset).
                self.classes[class_id].no_head = self.classes[class_id].yes_head;
                self.classes[class_id].yes_head = -1;
                self.classes[class_id].size = yes_size;
                self.classes[class_id].yes_size = 0;
            } else {
                // Moves the 'yes' subset to the new class (to the 'no' subset)
                self.classes[new_class_id].size = yes_size;
                self.classes[new_class_id].no_head = self.classes[class_id].yes_head;
                // Retains only the 'no' subset in the old class.
                self.classes[class_id].size = no_size;
                self.classes[class_id].yes_size = 0;
                self.classes[class_id].yes_head = -1;
            }

            // Updates the 'class_id' of all the elements we moved.
            let mut e = self.classes[new_class_id].no_head;
            loop {
                if e < 0 {
                    break;
                }
                self.elements[e as usize].class_id = new_class_id;
                e = self.elements[e as usize].next_element;
            }
            new_class_id as i32
        }
    }

    pub fn finalize_split<Q: Queue>(&mut self, queue: &mut Option<&mut Q>) {
        let visited_classes = self.visited_classes.clone();
        for visited_class in visited_classes {
            let new_class = self.split_refine(visited_class);
            if new_class != -1 && queue.as_ref().is_some() {
                queue.as_mut().unwrap().enqueue(new_class as StateId);
            }
        }
        self.visited_classes.clear();
        self.yes_counter += 1;
    }

    pub fn get_class_id(&self, element_id: usize) -> usize {
        self.elements[element_id].class_id
    }

    pub fn get_class_size(&self, class_id: usize) -> usize {
        self.classes[class_id].size
    }

    pub fn num_classes(&self) -> usize {
        self.classes.len()
    }

    pub fn iter(&self, class_id: usize) -> PartitionIterator {
        PartitionIterator::new(self, class_id)
    }
}

#[derive(Debug, Clone)]
pub struct Element {
    class_id: usize,
    yes: usize,
    next_element: i32,
    prev_element: i32,
}

impl Element {
    pub fn new() -> Self {
        Self {
            class_id: 0,
            yes: 0,
            next_element: 0,
            prev_element: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Class {
    size: usize,
    yes_size: usize,
    no_head: i32,
    yes_head: i32,
}

impl Class {
    pub fn new() -> Self {
        Self {
            size: 0,
            yes_size: 0,
            no_head: -1,
            yes_head: -1,
        }
    }
}

pub struct PartitionIterator<'a> {
    partition: &'a Partition,
    class_id: usize,
    last_element_id: Option<i32>,
}

impl<'a> PartitionIterator<'a> {
    pub fn new(partition: &'a Partition, class_id: usize) -> Self {
        PartitionIterator {
            partition,
            class_id,
            last_element_id: None,
        }
    }
}

impl<'a> Iterator for PartitionIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let new_element_id = match self.last_element_id {
            None => self.partition.classes[self.class_id].no_head,
            Some(e) => self.partition.elements[e as usize].next_element,
        };
        if new_element_id < 0 {
            None
        } else {
            self.last_element_id = Some(new_element_id);
            Some(new_element_id as usize)
        }
    }
}
