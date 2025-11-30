use super::*;

pub trait ServerSingleLinkLoad<T: Node>: SingleLink<T> {
    fn load_mut(&mut self, ctx: &ServerContext) -> NodeResult<&mut Self>;
    fn load_node(&self, ctx: &ServerContext) -> NodeResult<T>;
    fn load_node_mut(&mut self, ctx: &ServerContext) -> NodeResult<&mut T> {
        self.load_mut(ctx)?.get_mut()
    }
}

pub trait ServerMultipleLinkLoad<T: Node>: MultipleLink<T> {
    fn load_mut(&mut self, ctx: &ServerContext) -> NodeResult<&mut Self>;
    fn load_nodes(&self, ctx: &ServerContext) -> NodeResult<Vec<T>>;
    fn load_nodes_mut(&mut self, ctx: &ServerContext) -> NodeResult<&mut Vec<T>> {
        self.load_mut(ctx)?.get_mut()
    }
}

impl<T: Node + DeserializeOwned + Clone> ServerSingleLinkLoad<T> for Component<T> {
    fn load_mut(&mut self, ctx: &ServerContext) -> NodeResult<&mut Self> {
        if self.is_loaded() {
            return Ok(self);
        }

        let parent_id = self.parent_id();
        let children = ctx.load_linked::<T>(parent_id)?;

        if let Some(child) = children.first() {
            self.set_loaded(child.clone());
        } else {
            self.set_none();
        }

        Ok(self)
    }

    fn load_node(&self, ctx: &ServerContext) -> NodeResult<T> {
        if let Ok(value) = self.get() {
            return Ok(value.clone());
        }

        let parent_id = self.parent_id();
        let children = ctx.load_linked::<T>(parent_id)?;

        children
            .first()
            .cloned()
            .ok_or_else(|| NodeError::custom("No child found"))
    }
}

impl<T: Node + DeserializeOwned + Clone> ServerSingleLinkLoad<T> for Owned<T> {
    fn load_mut(&mut self, ctx: &ServerContext) -> NodeResult<&mut Self> {
        if self.is_loaded() {
            return Ok(self);
        }

        let parent_id = self.parent_id();
        let children = ctx.load_linked::<T>(parent_id)?;

        if let Some(child) = children.first() {
            self.set_loaded(child.clone());
        } else {
            self.set_none();
        }

        Ok(self)
    }

    fn load_node(&self, ctx: &ServerContext) -> NodeResult<T> {
        if let Ok(value) = self.get() {
            return Ok(value.clone());
        }

        let parent_id = self.parent_id();
        let children = ctx.load_linked::<T>(parent_id)?;

        children
            .first()
            .cloned()
            .ok_or_else(|| NodeError::custom("No child found"))
    }
}

impl<T: Node + DeserializeOwned + Clone> ServerSingleLinkLoad<T> for Ref<T> {
    fn load_mut(&mut self, ctx: &ServerContext) -> NodeResult<&mut Self> {
        if self.is_loaded() {
            return Ok(self);
        }

        let parent_id = self.parent_id();
        let children = ctx.load_linked::<T>(parent_id)?;

        if let Some(child) = children.first() {
            self.set_loaded(child.clone());
        } else {
            self.set_none();
        }

        Ok(self)
    }

    fn load_node(&self, ctx: &ServerContext) -> NodeResult<T> {
        if let Ok(value) = self.get() {
            return Ok(value.clone());
        }

        let parent_id = self.parent_id();
        let children = ctx.load_linked::<T>(parent_id)?;

        children
            .first()
            .cloned()
            .ok_or_else(|| NodeError::custom("No child found"))
    }
}

impl<T: Node + DeserializeOwned + Clone> ServerMultipleLinkLoad<T> for OwnedMultiple<T> {
    fn load_mut(&mut self, ctx: &ServerContext) -> NodeResult<&mut Self> {
        if self.is_loaded() {
            return Ok(self);
        }

        let parent_id = self.parent_id();
        let children = ctx.load_linked::<T>(parent_id)?;

        if !children.is_empty() {
            self.set_loaded(children);
        } else {
            self.set_none();
        }

        Ok(self)
    }

    fn load_nodes(&self, ctx: &ServerContext) -> NodeResult<Vec<T>> {
        if let Ok(values) = self.get() {
            return Ok(values.iter().cloned().collect());
        }

        let parent_id = self.parent_id();
        let children = ctx.load_linked::<T>(parent_id)?;

        Ok(children.iter().cloned().collect())
    }
}

impl<T: Node + DeserializeOwned + Clone> ServerMultipleLinkLoad<T> for RefMultiple<T> {
    fn load_mut(&mut self, ctx: &ServerContext) -> NodeResult<&mut Self> {
        if self.is_loaded() {
            return Ok(self);
        }

        let parent_id = self.parent_id();
        let children = ctx.load_linked::<T>(parent_id)?;

        if !children.is_empty() {
            self.set_loaded(children);
        } else {
            self.set_none();
        }

        Ok(self)
    }

    fn load_nodes(&self, ctx: &ServerContext) -> NodeResult<Vec<T>> {
        if let Ok(values) = self.get() {
            return Ok(values.iter().cloned().collect());
        }

        let parent_id = self.parent_id();
        let children = ctx.load_linked::<T>(parent_id)?;

        Ok(children.iter().cloned().collect())
    }
}
