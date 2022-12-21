use std::cell::RefCell;
use std::rc::Rc;
use crate::source::{ParseError, ParseET, Span};
use crate::tokens::tokens::Token;

#[derive(Debug, Clone)]
pub(crate) struct TokIter {
    vec: Rc<RefCell<Vec<Token>>>,
    pub(crate) index: usize,
}

impl TokIter {
    pub(crate) fn new(vec: Vec<Token>) -> Self{
        Self {
            vec: Rc::new(RefCell::new(vec)),
            index: 0,
        }
    }

    pub(crate) fn get(&self, index: usize) -> Result<Token, ParseError> {
        let v = self.vec.borrow();
        v.get(index).map(|t| t.clone())
            .ok_or_else(|| ParseET::EOF.at(self.nearest_point()
                .expect("something went really wrong when trying to get emergency loc when trying to error"))
                .when("trying to get token"))
    }

    pub(crate) fn nearest_point(&self) -> Result<Span, ParseError> {
        let v = self.vec.borrow();
        if v.len() == 0 {
            return Err(ParseET::EmptyInput.error())
        }
        else if self.index >= self.len() {
            Ok(v.last().unwrap().loc.clone())
        }
        else {
            Ok(v.get(self.index).unwrap().loc.clone())
        }
    }

    pub(crate) fn this(&self) -> Result<Token, ParseError>{
        self.get(self.index)
    }

    pub(crate) fn next(&mut self){
        self.index += 1;
    }

    pub(crate) fn len(&self) -> usize{
        self.vec.borrow().len()
    }

    pub(crate) fn left(&self) -> usize{
        self.vec.borrow().len() - self.index
    }

    pub(crate) fn insert(&mut self, t: Token) -> Result<(), ParseError>{
        let mut v = self.vec.borrow_mut();
        if self.index >= v.len() {
            return Err(ParseET::EOF.at(self.nearest_point()?).when("trying to insert Token"))
        }
        v.insert(self.index, t);
        self.index += 1;
        Ok(())
    }

    pub(crate) fn insert_stay(&mut self, t: Token) -> Result<(), ParseError>{
        let mut v = self.vec.borrow_mut();
        if self.index >= v.len() {
            return Err(ParseET::EOF.at(self.nearest_point()?).when("trying to insert Token"))
        }
        v.insert(self.index, t);
        Ok(())
    }

    pub(crate) fn push(&mut self, t: Token){
        self.index -= 1;
        self.vec.borrow_mut().push(t);
    }

    pub(crate) fn push_stay(&mut self, t: Token){
        self.vec.borrow_mut().push(t);
    }

    pub(crate) fn pop(&mut self) -> Result<Token, ParseError>{
        self.index -= 1;
        self.vec.borrow_mut().pop().ok_or(ParseET::EmptyInput.error().when("trying to pop Token"))
    }

    pub(crate) fn pop_stay(&mut self) -> Result<Token, ParseError>{
        self.vec.borrow_mut().pop().ok_or(ParseET::EmptyInput.error().when("trying to pop Token"))
    }

    pub(crate) fn remove(&mut self) -> Result<Token, ParseError>{
        let mut v = self.vec.borrow_mut();
        if v.len() >= self.index {
            return Err(ParseET::EOF.at(self.nearest_point()?).when("trying to insert Token"))
        }
        let r = Ok(v.remove(self.index));
        self.index -= 1;
        r
    }

    pub(crate) fn remove_stay(&mut self) -> Result<Token, ParseError>{
        let mut v = self.vec.borrow_mut();
        if v.len() >= self.index {
            return Err(ParseET::EOF.at(self.nearest_point()?).when("trying to insert Token"))
        }
        Ok(v.remove(self.index))
    }
}