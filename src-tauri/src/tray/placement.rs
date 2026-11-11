pub const GAP: i32 = 12;
pub const MIN_HEIGHT: u32 = 120;
pub const MAX_HEIGHT: u32 = 600;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Area {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

impl Area {
    const fn right(self) -> i32 {
        self.x.saturating_add(self.width)
    }

    const fn bottom(self) -> i32 {
        self.y.saturating_add(self.height)
    }

    const fn center(self) -> Point {
        Point {
            x: self.x.saturating_add(self.width / 2),
            y: self.y.saturating_add(self.height / 2),
        }
    }
}

#[must_use]
pub fn beside_icon(icon: Area, popup: Size, work: Area) -> Point {
    let center = icon.center();
    let preferred = match taskbar_edge(center, work) {
        Edge::Bottom => Point {
            x: center.x - popup.width / 2,
            y: icon.y.min(work.bottom()) - GAP - popup.height,
        },
        Edge::Top => Point {
            x: center.x - popup.width / 2,
            y: icon.bottom().max(work.y) + GAP,
        },
        Edge::Left => Point {
            x: icon.right().max(work.x) + GAP,
            y: center.y - popup.height / 2,
        },
        Edge::Right => Point {
            x: icon.x.min(work.right()) - GAP - popup.width,
            y: center.y - popup.height / 2,
        },
    };
    clamp(preferred, popup, work)
}

#[must_use]
pub fn corner(popup: Size, work: Area) -> Point {
    clamp(
        Point {
            x: work.right() - GAP - popup.width,
            y: work.bottom() - GAP - popup.height,
        },
        popup,
        work,
    )
}

#[must_use]
pub fn logical_height(content: u32) -> u32 {
    content.clamp(MIN_HEIGHT, MAX_HEIGHT)
}

#[must_use]
pub fn refit(popup: Area, height: i32, work: Area) -> Area {
    let height = height.min(work.height);
    let y = if popup.center().y >= work.center().y {
        popup.bottom() - height
    } else {
        popup.y
    };
    let size = Size {
        width: popup.width,
        height,
    };
    let point = clamp(Point { x: popup.x, y }, size, work);
    Area {
        x: point.x,
        y: point.y,
        width: popup.width,
        height,
    }
}

fn taskbar_edge(center: Point, work: Area) -> Edge {
    if center.y < work.y {
        Edge::Top
    } else if center.x < work.x {
        Edge::Left
    } else if center.x >= work.right() && center.y < work.bottom() {
        Edge::Right
    } else {
        Edge::Bottom
    }
}

fn clamp(point: Point, popup: Size, work: Area) -> Point {
    Point {
        x: point.x.min(work.right() - popup.width).max(work.x),
        y: point.y.min(work.bottom() - popup.height).max(work.y),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Area, GAP, MAX_HEIGHT, MIN_HEIGHT, Point, Size, beside_icon, corner, logical_height, refit,
    };

    const SCREEN: Area = Area {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
    };
    const POPUP: Size = Size {
        width: 360,
        height: 600,
    };

    fn icon(x: i32, y: i32) -> Area {
        Area {
            x,
            y,
            width: 24,
            height: 24,
        }
    }

    #[test]
    fn sits_above_an_icon_on_a_bottom_taskbar() {
        let work = Area {
            height: 1032,
            ..SCREEN
        };
        assert_eq!(
            beside_icon(icon(1500, 1044), POPUP, work),
            Point {
                x: 1500 + 12 - 180,
                y: 1032 - GAP - 600,
            }
        );
    }

    #[test]
    fn stays_inside_the_work_area_near_the_screen_edge() {
        let work = Area {
            height: 1032,
            ..SCREEN
        };
        let point = beside_icon(icon(1890, 1044), POPUP, work);
        assert_eq!(point.x, 1920 - 360);
    }

    #[test]
    fn sits_below_an_icon_on_a_top_taskbar() {
        let work = Area {
            y: 48,
            height: 1032,
            ..SCREEN
        };
        assert_eq!(beside_icon(icon(1500, 12), POPUP, work).y, 48 + GAP);
    }

    #[test]
    fn sits_beside_an_icon_on_a_side_taskbar() {
        let left = Area {
            x: 64,
            width: 1856,
            ..SCREEN
        };
        assert_eq!(beside_icon(icon(20, 900), POPUP, left).x, 64 + GAP);
        let right = Area {
            width: 1856,
            ..SCREEN
        };
        assert_eq!(
            beside_icon(icon(1876, 900), POPUP, right).x,
            1856 - GAP - 360
        );
        assert_eq!(beside_icon(icon(1876, 900), POPUP, right).y, 1080 - 600);
    }

    #[test]
    fn falls_back_to_the_bottom_right_corner() {
        let work = Area {
            height: 1032,
            ..SCREEN
        };
        assert_eq!(
            corner(POPUP, work),
            Point {
                x: 1920 - GAP - 360,
                y: 1032 - GAP - 600,
            }
        );
    }

    #[test]
    fn popups_larger_than_the_work_area_pin_to_its_origin() {
        let small = Area {
            x: 100,
            y: 50,
            width: 300,
            height: 400,
        };
        assert_eq!(corner(POPUP, small), Point { x: 100, y: 50 });
    }

    #[test]
    fn content_heights_stay_between_the_popup_bounds() {
        assert_eq!(logical_height(0), MIN_HEIGHT);
        assert_eq!(logical_height(412), 412);
        assert_eq!(logical_height(5000), MAX_HEIGHT);
    }

    #[test]
    fn refitting_above_a_bottom_taskbar_keeps_the_bottom_edge() {
        let work = Area {
            height: 1032,
            ..SCREEN
        };
        let popup = Area {
            x: 1332,
            y: 1032 - GAP - 600,
            width: 360,
            height: 600,
        };
        assert_eq!(
            refit(popup, 400, work),
            Area {
                y: 1032 - GAP - 400,
                height: 400,
                ..popup
            }
        );
    }

    #[test]
    fn refitting_below_a_top_taskbar_keeps_the_top_edge() {
        let work = Area {
            y: 48,
            height: 1032,
            ..SCREEN
        };
        let popup = Area {
            x: 1332,
            y: 48 + GAP,
            width: 360,
            height: 380,
        };
        assert_eq!(
            refit(popup, 560, work),
            Area {
                height: 560,
                ..popup
            }
        );
    }

    #[test]
    fn refitting_a_dragged_popup_follows_its_nearer_edge() {
        let popup = Area {
            x: 400,
            y: 100,
            width: 360,
            height: 600,
        };
        assert_eq!(refit(popup, 300, SCREEN).y, 100);
        let lower = Area { y: 400, ..popup };
        assert_eq!(refit(lower, 300, SCREEN).y, 700);
    }

    #[test]
    fn refitting_stays_inside_the_work_area() {
        let work = Area {
            x: 0,
            y: 0,
            width: 1920,
            height: 500,
        };
        let popup = Area {
            x: 1560,
            y: 200,
            width: 360,
            height: 288,
        };
        assert_eq!(
            refit(popup, 600, work),
            Area {
                y: 0,
                height: 500,
                ..popup
            }
        );
    }
}
