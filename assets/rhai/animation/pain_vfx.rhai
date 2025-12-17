animator.spawn_painter(`
    painter.alpha(1.0 - t);
    painter.scale_rect(t + 1.0);
    painter.circle(1.0);
    painter.paint();
`);
