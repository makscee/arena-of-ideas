// Scale the rect based on index
painter.scale_rect(1.2 + (index * 0.1));

// Draw colored hollow circle for status
painter.color(color);
painter.hollow(1.5);
painter.circle(1.0);
painter.paint();
