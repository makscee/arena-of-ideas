use super::*;

pub trait ClientSingleLinkLoad<T: Node>: SingleLink<T> {
    fn load_mut<'a>(&mut self, ctx: &ClientContext<'a>) -> NodeResult<&mut Self>;
    fn load_node<'a>(&self, ctx: &ClientContext<'a>) -> NodeResult<T>;
    fn load_mut_node<'a>(&mut self, ctx: &ClientContext<'a>) -> NodeResult<&mut T> {
        self.load_mut(ctx)?.get_mut()
    }
}

pub trait ClientMultipleLinkLoad<T: Node>: MultipleLink<T> {
    fn load_mut<'a>(&mut self, ctx: &ClientContext<'a>) -> NodeResult<&mut Self>;
    fn load_nodes<'a>(&self, ctx: &ClientContext<'a>) -> NodeResult<Vec<T>>;
    fn load_mut_nodes<'a>(&mut self, ctx: &ClientContext<'a>) -> NodeResult<&mut Vec<T>> {
        self.load_mut(ctx)?.get_mut()
    }
}

impl<T: ClientNode + Clone> ClientSingleLinkLoad<T> for Component<T> {
    fn load_mut<'a>(&mut self, ctx: &ClientContext<'a>) -> NodeResult<&mut Self> {
        if self.is_loaded() {
            return Ok(self);
        }
        let children = ctx.load_children_ref::<T>(self.parent_id())?;
        if let Some(child) = children.first() {
            self.set_loaded((*child).clone());
        } else {
            self.set_none();
        }

        Ok(self)
    }

    fn load_node<'a>(&self, ctx: &ClientContext<'a>) -> NodeResult<T> {
        if let Ok(value) = self.get() {
            return Ok(value.clone());
        }
        let children = ctx.load_children_ref::<T>(self.parent_id())?;
        children
            .first()
            .map(|child| (*child).clone())
            .ok_or_else(|| NodeError::custom("No child found"))
    }
}

impl<T: ClientNode + Clone> ClientSingleLinkLoad<T> for Owned<T> {
    fn load_mut<'a>(&mut self, ctx: &ClientContext<'a>) -> NodeResult<&mut Self> {
        if self.is_loaded() {
            return Ok(self);
        }
        let children = ctx.load_children_ref::<T>(self.parent_id())?;
        if let Some(child) = children.first() {
            self.set_loaded((*child).clone());
        } else {
            self.set_none();
        }

        Ok(self)
    }

    fn load_node<'a>(&self, ctx: &ClientContext<'a>) -> NodeResult<T> {
        if let Ok(value) = self.get() {
            return Ok(value.clone());
        }
        let children = ctx.load_children_ref::<T>(self.parent_id())?;
        children
            .first()
            .map(|child| (*child).clone())
            .ok_or_else(|| NodeError::custom("No child found"))
    }
}

impl<T: ClientNode + Clone> ClientSingleLinkLoad<T> for Ref<T> {
    fn load_mut<'a>(&mut self, ctx: &ClientContext<'a>) -> NodeResult<&mut Self> {
        if self.is_loaded() {
            return Ok(self);
        }
        let children = ctx.load_children_ref::<T>(self.parent_id())?;
        if let Some(child) = children.first() {
            self.set_loaded((*child).clone());
        } else {
            self.set_none();
        }

        Ok(self)
    }

    fn load_node<'a>(&self, ctx: &ClientContext<'a>) -> NodeResult<T> {
        if let Ok(value) = self.get() {
            return Ok(value.clone());
        }
        let children = ctx.load_children_ref::<T>(self.parent_id())?;
        children
            .first()
            .map(|child| (*child).clone())
            .ok_or_else(|| NodeError::custom("No child found"))
    }
}

impl<T: ClientNode + Clone> ClientMultipleLinkLoad<T> for OwnedMultiple<T> {
    fn load_mut<'a>(&mut self, ctx: &ClientContext<'a>) -> NodeResult<&mut Self> {
        if self.is_loaded() {
            return Ok(self);
        }
        let children = ctx.load_children_ref::<T>(self.parent_id())?;
        if !children.is_empty() {
            let loaded = children.iter().map(|child| (*child).clone()).collect();
            self.set_loaded(loaded);
        } else {
            self.set_none();
        }

        Ok(self)
    }

    fn load_nodes<'a>(&self, ctx: &ClientContext<'a>) -> NodeResult<Vec<T>> {
        if let Ok(values) = self.get() {
            return Ok(values.clone());
        }

        let children = ctx.load_children_ref::<T>(self.parent_id())?;

        Ok(children.into_iter().cloned().collect())
    }
}

impl<T: ClientNode + Clone> ClientMultipleLinkLoad<T> for RefMultiple<T> {
    fn load_mut<'a>(&mut self, ctx: &ClientContext<'a>) -> NodeResult<&mut Self> {
        if self.is_loaded() {
            return Ok(self);
        }

        let parent_id = self.parent_id();
        let children = ctx.load_children_ref::<T>(parent_id)?;

        if !children.is_empty() {
            let loaded = children.iter().map(|child| (*child).clone()).collect();
            self.set_loaded(loaded);
        } else {
            self.set_none();
        }

        Ok(self)
    }

    fn load_nodes<'a>(&self, ctx: &ClientContext<'a>) -> NodeResult<Vec<T>> {
        if let Ok(values) = self.get() {
            return Ok(values.clone());
        }
        let children = ctx.load_children_ref::<T>(self.parent_id())?;
        Ok(children.into_iter().cloned().collect())
    }
}
