animator.spawn_painter(`
    painter.alpha(1.0 - t_unit);
    painter.scale_rect(t_unit + 1.0);
    painter.circle(1.0);
    painter.paint();
`);
