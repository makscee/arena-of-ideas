// Status representation painter script
// Draws a status indicator with scaling based on index

// Scale the rect based on index
let scale = 1.2 + (index as f32 * 0.1);
painter.scale_rect(scale);

// Draw colored hollow circle for status
painter.color(255, 255, 255, 255);
painter.hollow(1.5);
painter.circle(1.0);
painter.paint();
