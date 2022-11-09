/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use std::ops::{Deref, DerefMut};

#[derive(Debug, Default, Clone, Copy)]
pub struct Ctx {
    pub root: bool,
    pub auto_reset: bool,
}

impl Ctx {
    pub fn root() -> Ctx {
        Ctx {
            root: true,
            auto_reset: true,
        }
    }

    pub fn child(&self, auto_reset: bool) -> Ctx {
        Ctx {
            root: false,
            auto_reset,
        }
    }
}

pub trait VisitorWithContext: swc_ecma_visit::VisitMut {
    fn get_ctx(&self) -> Ctx;
    fn set_ctx(&mut self, ctx: Ctx);
    fn reset_ctx(&mut self);

    fn with_ctx(&mut self, ctx: Ctx) -> WithCtx<'_, Self>
    where
        Self: Sized,
    {
        let orig_ctx = self.get_ctx();
        self.set_ctx(ctx);
        WithCtx {
            reducer: self,
            orig_ctx,
        }
    }

    fn with_child_ctx(&mut self) -> WithCtx<'_, Self>
    where
        Self: Sized,
    {
        self.with_ctx(self.get_ctx().child(true))
    }
}

//pub trait VisitMutWithContext: VisitorWithContext + swc_ecma_visit::VisitMut{}

pub struct WithCtx<'a, V>
where
    V: VisitorWithContext,
{
    pub reducer: &'a mut V,
    pub orig_ctx: Ctx,
}

impl<V: VisitorWithContext> Deref for WithCtx<'_, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.reducer
    }
}

impl<V: VisitorWithContext> DerefMut for WithCtx<'_, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.reducer
    }
}

impl<V: VisitorWithContext> Drop for WithCtx<'_, V> {
    fn drop(&mut self) {
        let auto_reset = self.reducer.get_ctx().auto_reset;
        self.reducer.set_ctx(self.orig_ctx);
        if self.reducer.get_ctx().root & auto_reset {
            self.reducer.reset_ctx();
        }
    }
}
