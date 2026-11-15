use anyhow::Result;
use gpui::{App, Context, Task, Window};
use ropey::Rope;
use std::rc::Rc;
use crate::input::{EditorMode, FindReferences, InputBaseState};

pub trait ReferencesProvider {
    fn references(&self, text: &Rope, offset: usize, window: &mut Window, cx: &mut App) -> Task<Result<Vec<lsp_types::Location>>>;
}
pub type ShowReferencesHandler = Rc<dyn Fn(&[lsp_types::Location], Option<&str>, &mut Window, &mut App)>;

impl InputBaseState<EditorMode> {
    pub(crate) fn on_action_find_references(&mut self, _: &FindReferences, window: &mut Window, cx: &mut Context<Self>) {
        let (Some(provider),Some(show))=(self.extras.lsp.references_provider.clone(),self.extras.lsp.show_references.clone()) else {return};
        let offset=self.cursor();
        let text=self.text.clone();
        let response=provider.references(&text,offset,window,cx);
        let blur=cx.on_blur(&self.focus_handle,window,|editor,_,_| editor.extras.lsp._references_task=Task::ready(Ok(())));
        self.extras.lsp._references_task=cx.spawn_in(window,async move |editor,cx| {
            let _blur=blur;
            let response=response.await;
            let error=response.as_ref().err().map(ToString::to_string);
            let locations=response.unwrap_or_default();
            editor.update_in(cx,|editor,window,cx| {
                if editor.cursor()==offset && editor.text==text && editor.focus_handle.is_focused(window) {show(&locations,error.as_deref(),window,cx);}
            })?;
            Ok(())
        });
    }
}
