/* Parameters */
seal_text_length = 15;
seal_padding = 2;
seal_height = 2;

wall_thickness = 1;
handle_height = 25;

/* Model */
resize([seal_text_length, seal_text_length, seal_height])
    linear_extrude(height = seal_height) 
        import("test.svg", center = true);

translate([0, 0, seal_height / 2]) difference() {
    cube([seal_text_length + seal_padding + wall_thickness * 2, seal_text_length + seal_padding + wall_thickness * 2, seal_height - 1], center = true);
    cube([seal_text_length + seal_padding, seal_text_length + seal_padding, seal_height - 1], center = true);
};

translate([0, 0, -handle_height / 2 + 1 / 2]) cube([seal_text_length + seal_padding + wall_thickness * 2, seal_text_length + seal_padding + wall_thickness * 2, handle_height], center=true);