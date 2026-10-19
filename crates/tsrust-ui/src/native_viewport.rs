//! Window scrolling shares logical coordinates with layout, input and accessibility.
#[derive(Default)]
pub(super) struct Viewport {
    pub size: Option<(f64, f64)>,
    pub content: (f64, f64),
    pub offset: (f64, f64),
    pub reveal_focus: bool,
}
impl Viewport {
    pub fn maximum(&self) -> (f64, f64) {
        self.size.map(|size| ((self.content.0-size.0).max(0.), (self.content.1-size.1).max(0.))).unwrap_or((0.,0.))
    }
    pub fn scroll_to(&mut self, x: f64, y: f64) -> bool {
        let max=self.maximum();
        let next=(if x.is_finite(){x.clamp(0.,max.0)}else{self.offset.0},if y.is_finite(){y.clamp(0.,max.1)}else{self.offset.1});
        let changed=next!=self.offset;self.offset=next;changed
    }
    pub fn scroll_by(&mut self, x:f64, y:f64)->bool {self.scroll_to(self.offset.0+x,self.offset.1+y)}
    /// Bounds are in unscrolled logical coordinates. Large controls align at their start.
    pub fn reveal(&mut self,bounds:accesskit::Rect)->bool {
        let Some(size)=self.size else{return false;};
        fn axis(start:f64,end:f64,offset:f64,size:f64)->f64 {
            if start<offset || end-start>size {start}else if end>offset+size {end-size}else{offset}
        }
        self.scroll_to(axis(bounds.x0,bounds.x1,self.offset.0,size.0),axis(bounds.y0,bounds.y1,self.offset.1,size.1))
    }
    pub fn accessibility(&self,node:&mut accesskit::Node,scale:f64) {
        let Some(size)=self.size else{return;};let max=self.maximum();
        node.set_bounds(accesskit::Rect::new(0.,0.,size.0*scale,size.1*scale));node.set_clips_children();
        node.set_scroll_x(self.offset.0*scale);node.set_scroll_y(self.offset.1*scale);
        node.set_scroll_x_min(0.);node.set_scroll_y_min(0.);node.set_scroll_x_max(max.0*scale);node.set_scroll_y_max(max.1*scale);
        if max.0>0. {node.add_action(accesskit::Action::ScrollLeft);node.add_action(accesskit::Action::ScrollRight);}
        if max.1>0. {node.add_action(accesskit::Action::ScrollUp);node.add_action(accesskit::Action::ScrollDown);}
        if max!=(0.,0.) {node.add_action(accesskit::Action::SetScrollOffset);node.add_child_action(accesskit::Action::ScrollIntoView);}
    }
}
