use annealer::{Config, FormatError, Language, Profile, format};

fn html(input: &str) -> String {
    run(input, Language::Html)
}

fn vue(input: &str) -> String {
    run(input, Language::Vue)
}

/// Formats and asserts that a second pass is a no-op.
fn run(input: &str, language: Language) -> String {
    let config = Config::for_language(language);
    let once = format(input, &config).unwrap();
    let twice = format(&once, &config).unwrap();
    assert_eq!(once, twice, "formatting is not idempotent");
    once
}

fn sfc(template: &str) -> String {
    format!("<template>\n{template}\n</template>\n")
}

mod html_profile {
    use super::*;

    #[test]
    fn orders_by_semantic_group() {
        assert_eq!(
            html(
                r#"<img alt="Logo" onload="f()" data-x="1" width="10" src="a.png" class="c" id="i">"#
            ),
            r#"<img id="i" class="c" alt="Logo" src="a.png" width="10" data-x="1" onload="f()" />"#
        );
    }

    #[test]
    fn orders_aria_by_role_name_description_property_state() {
        assert_eq!(
            html(
                r#"<button aria-expanded="false" aria-controls="m" aria-describedby="d" aria-label="Menu" role="button">"#
            ),
            r#"<button role="button" aria-label="Menu" aria-describedby="d" aria-controls="m" aria-expanded="false">"#
        );
    }

    #[test]
    fn sorts_data_attributes_alphabetically() {
        assert_eq!(
            html(r#"<div data-z="1" data-a="2" data-m="3">"#),
            r#"<div data-a="2" data-m="3" data-z="1">"#
        );
    }

    #[test]
    fn separates_key_value_dimensions_and_state() {
        assert_eq!(
            html(r#"<input disabled height="20" value="v" width="80" type="image" name="n">"#),
            r#"<input type="image" name="n" value="v" width="80" height="20" disabled />"#
        );
    }

    #[test]
    fn treats_meta_content_as_key_value() {
        assert_eq!(
            html(r#"<meta content="A page" id="m" name="description">"#),
            r#"<meta id="m" name="description" content="A page" />"#
        );
    }

    #[test]
    fn keeps_source_order_within_fallback_group() {
        assert_eq!(
            html(r#"<input required readonly disabled>"#),
            r#"<input required readonly disabled />"#
        );
    }

    #[test]
    fn preserves_case_quotes_and_unquoted_values() {
        assert_eq!(
            html("<DIV Title='x' ID=main hidden>"),
            "<DIV ID=main Title='x' hidden>"
        );
    }

    #[test]
    fn leaves_raw_text_comments_and_doctype_alone() {
        let input = concat!(
            "<!DOCTYPE html>\n",
            "<!-- <a title=\"t\" id=\"i\"> -->\n",
            "<script>if (a <b && c) document.write('<a title=\"t\" id=\"i\">')</script>\n",
            "<textarea><a title=\"t\" id=\"i\"></textarea>\n",
        );
        assert_eq!(html(input), input);
    }

    #[test]
    fn puts_closing_bracket_of_multiline_tag_on_its_own_line() {
        assert_eq!(
            html("  <img\n    title=\"t\"\n    id=\"i\" />"),
            "  <img\n    id=\"i\"\n    title=\"t\"\n  />"
        );
    }

    #[test]
    fn puts_each_attribute_on_its_own_line_in_multiline_tags() {
        assert_eq!(
            html("<a\n  title=\"t\" href=\"/\"\n  id=\"i\">x</a>"),
            "<a\n  id=\"i\"\n  title=\"t\"\n  href=\"/\"\n>x</a>"
        );
    }

    #[test]
    fn keeps_closing_bracket_of_single_line_tags_on_that_line() {
        assert_eq!(
            html("<a href=\"/\" title=\"t\"\n  >x</a><br\n/><p\n>y</p>"),
            // `<p>` spans two lines in the input, so its content is broken too.
            "<a title=\"t\" href=\"/\">x</a><br /><p>\n  y\n</p>"
        );
    }

    #[test]
    fn keeps_crlf_line_endings() {
        assert_eq!(
            html("<a\r\n  title=\"t\"\r\n  id=\"i\">x</a>"),
            "<a\r\n  id=\"i\"\r\n  title=\"t\"\r\n>x</a>"
        );
    }

    #[test]
    fn does_not_hyphenate_or_shorten_in_html() {
        assert_eq!(
            html(r#"<my-el fooBar="1" v-bind:x="y">"#),
            r#"<my-el fooBar="1" v-bind:x="y">"#
        );
    }

    #[test]
    fn orders_style_block_properties() {
        assert_eq!(
            html("<style>\n  a { color: red; display: block; }\n</style>"),
            "<style>\n  a {\n    display: block;\n    color: red;\n  }\n</style>"
        );
    }
}

mod vue_profile {
    use super::*;

    #[test]
    fn follows_vue_attributes_order_tiers() {
        assert_eq!(
            vue(&sfc(
                r#"<Comp v-html="h" @click="c" title="t" v-focus v-model="m" ref="r" id="i" v-once v-if="x" v-for="a in b" is="X">"#
            )),
            sfc(
                r#"<Comp is="X" v-for="a in b" v-if="x" v-once id="i" ref="r" v-model="m" v-focus title="t" @click="c" v-html="h">"#
            )
        );
    }

    #[test]
    fn clusters_static_and_bound_class_static_first() {
        assert_eq!(
            vue(&sfc(r#"<div :class="b" title="t" class="a">"#)),
            sfc(r#"<div class="a" :class="b" title="t">"#)
        );
    }

    #[test]
    fn keeps_source_order_inside_non_class_clusters() {
        // For `style`, the later declaration wins, so order is behavior.
        assert_eq!(
            vue(&sfc(r#"<div :style="s" id="i" style="color: red">"#)),
            sfc(r#"<div id="i" :style="s" style="color: red">"#)
        );
    }

    #[test]
    fn does_not_cluster_v_model_with_value() {
        assert_eq!(
            vue(&sfc(r#"<input value="x" v-model="y">"#)),
            sfc(r#"<input v-model="y" value="x" />"#)
        );
    }

    #[test]
    fn never_moves_attributes_across_spread() {
        assert_eq!(
            vue(&sfc(r#"<Comp :title="t" v-bind="obj" @click="c" id="i">"#)),
            sfc(r#"<Comp :title="t" v-bind="obj" id="i" @click="c">"#)
        );
    }

    #[test]
    fn hoists_spread_safe_directives_across_spread() {
        assert_eq!(
            vue(&sfc(
                r#"<Comp title="t" v-bind="obj" v-if="ok" v-for="x in xs">"#
            )),
            sfc(r#"<Comp v-for="x in xs" v-if="ok" title="t" v-bind="obj">"#)
        );
    }

    #[test]
    fn normalizes_directive_shorthand() {
        assert_eq!(
            vue(&sfc(r#"<div v-on:click.stop="c" v-bind:title="t">"#)),
            sfc(r#"<div :title="t" @click.stop="c">"#)
        );
    }

    #[test]
    fn hyphenates_component_props_only() {
        assert_eq!(
            vue(&sfc(
                r#"<MyComp :itemCount="n" maxLength="3" @updateValue="u"><input maxLength="3"><svg><clipPath clipPathUnits="x"/></svg></MyComp>"#
            )),
            sfc(
                r#"<MyComp :item-count="n" max-length="3" @updateValue="u"><input maxLength="3" /><svg><clipPath clipPathUnits="x" /></svg></MyComp>"#
            )
        );
    }

    #[test]
    fn skips_interpolations() {
        let input = sfc("<p>{{ a<b ? '<i title=\"t\" id=\"i\">' : '' }}</p>");
        assert_eq!(vue(&input), input);
    }

    #[test]
    fn scans_nested_templates_and_formats_style() {
        let input = concat!(
            "<template>\n",
            "  <List title=\"t\" v-if=\"ok\">\n",
            "    <template #item=\"{ x }\"><b title=\"t\" id=\"i\">{{ x }}</b></template>\n",
            "  </List>\n",
            "</template>\n",
            "\n",
            "<style scoped lang=\"scss\">\n",
            ".a { color: red; position: absolute; }\n",
            "</style>\n",
        );
        let expected = concat!(
            "<template>\n",
            "  <List v-if=\"ok\" title=\"t\">\n",
            "    <template #item=\"{ x }\"><b id=\"i\" title=\"t\">{{ x }}</b></template>\n",
            "  </List>\n",
            "</template>\n",
            "\n",
            "<style scoped lang=\"scss\">\n",
            ".a {\n  position: absolute;\n  color: red;\n}\n",
            "</style>\n",
        );
        assert_eq!(vue(input), expected);
    }

    #[test]
    fn leaves_script_custom_blocks_and_non_html_templates_alone() {
        let input = concat!(
            "<script setup lang=\"ts\">\nconst s = '<a title=\"t\" id=\"i\">';\n</script>\n",
            "<i18n lang=\"json\">{ \"a\": \"<a title='t' id='i'>\" }</i18n>\n",
            "<template lang=\"pug\">\na(title=\"t\" id=\"i\")\n</template>\n",
            "<style lang=\"stylus\">\na { color: red; display: block; }\n</style>\n",
        );
        assert_eq!(vue(input), input);
    }
}

mod content_newline {
    use super::*;

    #[test]
    fn breaks_content_of_elements_with_multiline_start_tags() {
        assert_eq!(
            vue(&sfc("<div\n  title=\"t\" id=\"i\">Hello {{ name }}</div>")),
            sfc("<div\n  id=\"i\"\n  title=\"t\"\n>\n  Hello {{ name }}\n</div>")
        );
    }

    #[test]
    fn breaks_content_of_elements_with_multiline_content() {
        assert_eq!(
            vue(&sfc("  <section>text\n    more</section>")),
            sfc("  <section>\n    text\n    more\n  </section>")
        );
    }

    #[test]
    fn leaves_single_line_and_empty_elements_alone() {
        let input = sfc("<p>one</p>\n<p v-if=\"a\"><b>x</b></p>\n<textarea>\n</textarea>");
        assert_eq!(vue(&input), input);
        let input = "<div>\n</div>\n<div\n  id=\"i\"\n>\n</div>";
        assert_eq!(html(input), input);
    }

    #[test]
    fn collapses_empty_lines_around_content() {
        assert_eq!(
            vue(&sfc("<MyComp>\n\n\n  slot\n\n</MyComp>")),
            sfc("<MyComp>\n  slot\n</MyComp>")
        );
    }

    #[test]
    fn leaves_pre_textarea_and_inline_elements_alone() {
        let input = sfc(concat!(
            "<pre\n  class=\"x\"\n>  keep  </pre>\n",
            "<span\n  class=\"x\"\n>a <div>b\nc</div></span>\n",
            "<a\n  href=\"/\"\n>x</a>",
        ));
        assert_eq!(vue(&input), input);
    }

    #[test]
    fn breaks_content_around_nested_multiline_tags_idempotently() {
        assert_eq!(
            vue(&sfc(
                "  <p>Inline <span\n    class=\"a\">kept</span> here</p>"
            )),
            sfc("  <p>\n    Inline <span\n    class=\"a\"\n    >kept</span> here\n  </p>")
        );
    }

    #[test]
    fn handles_implicitly_closed_html_elements() {
        assert_eq!(html("<ul><li>a<li>b\n</ul>"), "<ul>\n  <li>a<li>b\n</ul>");
    }

    #[test]
    fn respects_disable_directives() {
        let input = sfc("<!-- annealer-disable-next-line -->\n<div\n  id=\"i\">x</div>");
        assert_eq!(vue(&input), input);
    }

    #[test]
    fn uses_the_profile_ignore_list_and_empty_line_option() {
        let profile = Profile::from_yaml(
            "schemaVersion: 1\nname: x\nlayout:\n  contentNewline:\n    ignore: [MyComp]\n    allowEmptyLines: true\ngroups:\n  - name: rest\n    fallback: true\n",
        )
        .unwrap();
        let config = Config::new(Language::Vue, profile);
        let input = sfc("<MyComp>a\nb</MyComp>\n<pre>a\nb</pre>\n<div>\n\n  c\n\n</div>");
        assert_eq!(
            format(&input, &config).unwrap(),
            sfc("<MyComp>a\nb</MyComp>\n<pre>\n  a\nb\n</pre>\n<div>\n\n  c\n\n</div>")
        );
    }

    #[test]
    fn keeps_crlf_line_endings() {
        assert_eq!(
            html("<div\r\n  id=\"i\">x</div>"),
            "<div\r\n  id=\"i\"\r\n>\r\n  x\r\n</div>"
        );
    }
}

mod style_attribute {
    use super::*;

    fn with_style_attribute(input: &str) -> String {
        let yaml = include_str!("../profiles/html.yaml").replace(
            "  vendorPrefix: start\n",
            "  vendorPrefix: start\n  styleAttribute: true\n",
        );
        let config = Config::new(Language::Html, Profile::from_yaml(&yaml).unwrap());
        let once = format(input, &config).unwrap();
        assert_eq!(format(&once, &config).unwrap(), once, "not idempotent");
        once
    }

    #[test]
    fn is_off_by_default() {
        let input = r#"<p style="color: red; display: block">"#;
        assert_eq!(html(input), input);
    }

    #[test]
    fn orders_declarations_and_vendor_prefixes() {
        assert_eq!(
            with_style_attribute(
                r#"<p style="color: red; box-shadow:none; -webkit-box-shadow: none; display: block">"#
            ),
            r#"<p style="-webkit-box-shadow: none; display: block; box-shadow: none; color: red">"#
        );
    }

    #[test]
    fn keeps_trailing_semicolon_choice() {
        assert_eq!(
            with_style_attribute(r#"<p style="color: red; display: block;">"#),
            r#"<p style="display: block; color: red;">"#
        );
    }

    #[test]
    fn uses_the_opposite_quote_for_css_strings() {
        assert_eq!(
            with_style_attribute(r#"<p style='font-family: "A B"; position: absolute'>"#),
            r#"<p style='position: absolute; font-family: "A B"'>"#
        );
        assert_eq!(
            with_style_attribute(r#"<p style="font-family: 'A B'; position: absolute">"#),
            r#"<p style="position: absolute; font-family: 'A B'">"#
        );
    }

    #[test]
    fn leaves_template_syntax_comments_and_bound_styles_alone() {
        let input = concat!(
            r#"<p style="color: {{ c }}; display: block">"#,
            r#"<p style="color: red; /* x */ display: block">"#,
            r#"<p style="color: red; display:">"#,
            r#"<p :style="{ color: c, display: d }">"#,
            r#"<p style=color:red>"#,
        );
        assert_eq!(with_style_attribute(input), input);
    }
}

mod self_closing {
    use super::*;

    #[test]
    fn self_closes_void_elements_in_html() {
        assert_eq!(
            html(r#"<br><img src="a.png"><hr/><input type="text" ><BR>"#),
            r#"<br /><img src="a.png" /><hr /><input type="text" /><BR />"#
        );
    }

    #[test]
    fn keeps_empty_normal_elements_in_html() {
        let input = "<div></div><my-el></my-el><span />";
        assert_eq!(html(input), input);
    }

    #[test]
    fn puts_self_closing_bracket_of_multiline_void_on_its_own_line() {
        assert_eq!(
            html("<img\n  alt=\"a\"\n  src=\"b\">"),
            "<img\n  alt=\"a\"\n  src=\"b\"\n/>"
        );
    }

    #[test]
    fn self_closes_empty_elements_and_components_in_vue() {
        assert_eq!(
            vue(&sfc(
                "<div></div><MyComp :a=\"b\">\n</MyComp><p> x </p><Link></Link><link>"
            )),
            sfc("<div /><MyComp :a=\"b\" /><p> x </p><Link /><link />")
        );
    }

    #[test]
    fn keeps_significant_whitespace_and_mismatched_end_tags() {
        let input = sfc("<textarea> </textarea><pre>\n</pre><MyComp></mycomp>");
        assert_eq!(vue(&input), input);
        assert_eq!(vue(&sfc("<textarea></textarea>")), sfc("<textarea />"));
    }

    #[test]
    fn self_closes_empty_nested_templates() {
        assert_eq!(
            vue(&sfc(
                "<List><template #empty></template><template #item><b></b></template></List>"
            )),
            sfc("<List><template #empty /><template #item><b /></template></List>")
        );
    }

    #[test]
    fn leaves_sfc_top_level_blocks_alone() {
        let input =
            "<template>\n  <div />\n</template>\n<script setup></script>\n<style></style>\n";
        assert_eq!(vue(input), input);
    }

    #[test]
    fn expands_self_closing_with_never() {
        let profile = Profile::from_yaml(
            "schemaVersion: 1\nname: x\nlayout:\n  selfClosing:\n    void: never\n    normal: never\n    component: never\ngroups:\n  - name: rest\n    fallback: true\n",
        )
        .unwrap();
        let config = Config::new(Language::Vue, profile);
        assert_eq!(
            format(
                &sfc("<MyComp a=\"b\" /><div/><br /><img src=\"a\"/>"),
                &config
            )
            .unwrap(),
            sfc("<MyComp a=\"b\"></MyComp><div></div><br><img src=\"a\">")
        );
    }
}

mod directives {
    use super::*;

    #[test]
    fn disable_next_line_leaves_tags_on_that_line_alone() {
        let input = concat!(
            "<!-- annealer-disable-next-line: third-party snippet -->\n",
            "<a title=\"t\" id=\"i\"><br></a>\n",
            "<a title=\"t\" id=\"i\"><br></a>\n",
        );
        let expected = concat!(
            "<!-- annealer-disable-next-line: third-party snippet -->\n",
            "<a title=\"t\" id=\"i\"><br></a>\n",
            "<a id=\"i\" title=\"t\"><br /></a>\n",
        );
        assert_eq!(html(input), expected);
    }

    #[test]
    fn disable_next_line_covers_a_whole_multiline_tag() {
        let input = sfc(concat!(
            "<!-- annealer-disable-next-line -->\n",
            "<Comp\n  title=\"t\" v-if=\"x\">\n",
            "</Comp>",
        ));
        assert_eq!(vue(&input), input);
    }

    #[test]
    fn disable_and_enable_bound_a_range() {
        let input = concat!(
            "<a title=\"t\" id=\"1\">\n",
            "<!-- annealer-disable -->\n",
            "<a title=\"t\" id=\"2\">\n",
            "<style>a { color: red; display: block; }</style>\n",
            "<!-- annealer-enable -->\n",
            "<a title=\"t\" id=\"3\">\n",
        );
        let expected = concat!(
            "<a id=\"1\" title=\"t\">\n",
            "<!-- annealer-disable -->\n",
            "<a title=\"t\" id=\"2\">\n",
            "<style>a { color: red; display: block; }</style>\n",
            "<!-- annealer-enable -->\n",
            "<a id=\"3\" title=\"t\">\n",
        );
        assert_eq!(html(input), expected);
    }

    #[test]
    fn disable_without_enable_covers_the_rest() {
        let input = "<!-- annealer-disable -->\n<a title=\"t\" id=\"i\"><br><div></div>\n";
        assert_eq!(html(input), input);
        let input = "<!-- annealer-disable -->\n<template>\n<a title=\"t\" id=\"i\"><div></div>\n</template>\n";
        assert_eq!(vue(input), input);
    }

    #[test]
    fn ignores_ordinary_comments() {
        assert_eq!(
            html("<!-- see annealer-disable -->\n<a title=\"t\" id=\"i\">"),
            "<!-- see annealer-disable -->\n<a id=\"i\" title=\"t\">"
        );
    }

    #[test]
    fn rejects_unknown_markup_directives() {
        let error = format(
            "<p>\n  <!-- annealer-disabel -->",
            &Config::for_language(Language::Html),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            FormatError::Directive {
                line: 2,
                column: 3,
                ..
            }
        ));
    }

    #[test]
    fn css_disable_next_line_keeps_declaration_verbatim_and_in_place() {
        assert_eq!(
            run(
                concat!(
                    "a {\n",
                    "  color: red;\n",
                    "  /* annealer-disable-next-line: legacy order */\n",
                    "  -webkit-transition:x;\n",
                    "  transition: x;\n",
                    "  display: block;\n",
                    "}\n",
                ),
                Language::Css
            ),
            concat!(
                "a {\n",
                "  color: red;\n",
                "  /* annealer-disable-next-line: legacy order */\n",
                "  -webkit-transition:x;\n",
                "  display: block;\n",
                "  transition: x;\n",
                "}\n",
            )
        );
    }

    #[test]
    fn css_disable_range_keeps_rules_verbatim() {
        assert_eq!(
            run(
                concat!(
                    "a { color: red; display: block; }\n",
                    "/* annealer-disable */\n",
                    "b { color: red;   display: block; }\n",
                    "/* annealer-enable */\n",
                    "c { color: red; display: block; }\n",
                ),
                Language::Css
            ),
            concat!(
                "a {\n  display: block;\n  color: red;\n}\n",
                "/* annealer-disable */\n",
                "b { color: red;   display: block; }\n",
                "/* annealer-enable */\n",
                "c {\n  display: block;\n  color: red;\n}\n",
            )
        );
    }

    #[test]
    fn css_enable_comment_stays_at_the_range_boundary() {
        assert_eq!(
            run(
                "a {\n  /* annealer-disable */\n  top:0;\n  -webkit-x:1;\n  /* annealer-enable */\n  color: red; display: block;\n}\n",
                Language::Scss
            ),
            "a {\n  /* annealer-disable */\n  top:0;\n  -webkit-x:1;\n  /* annealer-enable */\n  display: block;\n  color: red;\n}\n"
        );
    }

    #[test]
    fn css_disable_at_top_leaves_the_file_unchanged() {
        let input = "/* annealer-disable */\na { color: red;   display: block; }\n\n\nb{top:0}\n";
        assert_eq!(run(input, Language::Css), input);
    }

    #[test]
    fn css_directives_work_in_style_blocks() {
        assert_eq!(
            vue(concat!(
                "<style>\n",
                ".a {\n",
                "  /* annealer-disable-next-line */\n",
                "  color:red;\n",
                "  display: block;\n",
                "}\n",
                "</style>\n",
            )),
            concat!(
                "<style>\n",
                ".a {\n",
                "  /* annealer-disable-next-line */\n",
                "  color:red;\n",
                "  display: block;\n",
                "}\n",
                "</style>\n",
            )
        );
    }

    #[test]
    fn rejects_unknown_css_directives() {
        let error = format(
            "a { color: red; /* annealer-disabel */ }\n",
            &Config::for_language(Language::Css),
        )
        .unwrap_err();
        assert!(matches!(error, FormatError::Stylesheet { .. }));
    }
}

mod stylesheets {
    use super::*;

    #[test]
    fn delegates_property_order_to_malva() {
        assert_eq!(
            run("a { color: red; display: block; }\n", Language::Css),
            "a {\n  display: block;\n  color: red;\n}\n"
        );
    }

    #[test]
    fn moves_vendor_prefixed_declarations_first_alphabetically() {
        assert_eq!(
            run(
                "a { -webkit-transition: x; transition: x; -moz-appearance: none; appearance: none; color: red; -webkit-appearance: none; }\n",
                Language::Css
            ),
            "a {\n  -moz-appearance: none;\n  -webkit-appearance: none;\n  -webkit-transition: x;\n  appearance: none;\n  color: red;\n  transition: x;\n}\n"
        );
    }

    #[test]
    fn keeps_vendor_prefixes_within_their_run() {
        // A nested rule splits the runs, as it does for malva's own sorting.
        assert_eq!(
            run(
                "a {\n  -webkit-user-select: none;\n  display: block;\n  &:hover { -moz-opacity: 1; color: red; }\n  --x: 1;\n  -ms-zoom: 1;\n}\n",
                Language::Scss
            ),
            "a {\n  -webkit-user-select: none;\n  display: block;\n  &:hover {\n    -moz-opacity: 1;\n    color: red;\n  }\n  -ms-zoom: 1;\n  --x: 1;\n}\n"
        );
    }

    #[test]
    fn leaves_runs_with_comments_alone() {
        let input = "a {\n  transition: x; /* modern */\n  -webkit-transition: x;\n}\n";
        let config = Config::for_language(Language::Css);
        let output = format(input, &config).unwrap();
        assert!(output.find("  transition").unwrap() < output.find("-webkit-transition").unwrap());
    }

    #[test]
    fn orders_vendor_prefixes_in_style_blocks() {
        assert_eq!(
            vue("<style>\n.a { box-shadow: none; -webkit-box-shadow: none; }\n</style>\n"),
            "<style>\n.a {\n  -webkit-box-shadow: none;\n  box-shadow: none;\n}\n</style>\n"
        );
    }
}

mod less_and_sass {
    use super::*;

    #[test]
    fn formats_less() {
        assert_eq!(
            run(
                "@w: 1px;\na { color: red; -webkit-box-shadow: none; .m(); display: block; width: @w; }\n",
                Language::Less
            ),
            "@w: 1px;\na {\n  -webkit-box-shadow: none;\n  color: red;\n  .m();\n  display: block;\n  width: @w;\n}\n"
        );
    }

    #[test]
    fn formats_indented_sass() {
        assert_eq!(
            run(
                "a\n  color: red\n  -webkit-transition: x\n  display: block\n  &:hover\n    top: 0\n    position: absolute\n",
                Language::Sass
            ),
            "a\n  -webkit-transition: x\n  display: block\n  color: red\n  &:hover\n    position: absolute\n    top: 0\n"
        );
    }

    #[test]
    fn applies_directives_in_indented_sass() {
        let input = concat!(
            "a\n",
            "  color: red\n",
            "  /* annealer-disable-next-line */\n",
            "  -webkit-x:  1\n",
            "  display: block\n",
            "  /* annealer-disable */\n",
            "  z-index: 1\n",
            "  /* annealer-enable */\n",
            "  top: 0\n",
            "  position: absolute\n",
        );
        assert_eq!(
            run(input, Language::Sass),
            concat!(
                "a\n",
                "  color: red\n",
                "  /* annealer-disable-next-line */\n",
                "  -webkit-x:  1\n",
                "  display: block\n",
                "  /* annealer-disable */\n",
                "  z-index: 1\n",
                "  /* annealer-enable */\n",
                "  position: absolute\n",
                "  top: 0\n",
            )
        );
    }

    #[test]
    fn applies_directives_in_less() {
        let input = "a {\n  color: red;\n  /* annealer-disable-next-line */\n  -webkit-x:1;\n  display: block;\n}\n";
        assert_eq!(run(input, Language::Less), input);
    }

    #[test]
    fn formats_less_and_sass_style_blocks() {
        assert_eq!(
            vue(concat!(
                "<style lang=\"less\">\n.a { color: red; display: block; }\n</style>\n",
                "<style lang=\"sass\">\n.a\n  color: red\n  display: block\n</style>\n",
            )),
            concat!(
                "<style lang=\"less\">\n.a {\n  display: block;\n  color: red;\n}\n</style>\n",
                "<style lang=\"sass\">\n.a\n  display: block\n  color: red\n</style>\n",
            )
        );
    }
}

mod errors_and_profiles {
    use super::*;

    #[test]
    fn reports_unterminated_attribute_value() {
        let error = format(
            "<p>\n  <a title=\"oops>",
            &Config::for_language(Language::Html),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            FormatError::UnterminatedAttributeValue { line: 2, column: 6 }
        ));
    }

    #[test]
    fn reports_unterminated_tag() {
        let error = format("<a title=\"t\"", &Config::for_language(Language::Html)).unwrap_err();
        assert!(matches!(
            error,
            FormatError::UnterminatedTag { line: 1, column: 1 }
        ));
    }

    #[test]
    fn applies_custom_profile() {
        let profile = Profile::from_yaml(
            "schemaVersion: 1\nname: custom\ngroups:\n  - name: rest\n    fallback: true\n    sort: alphabetical\n  - name: ids\n    match: [id]\n",
        )
        .unwrap();
        let config = Config::new(Language::Html, profile);
        assert_eq!(
            format(r#"<a id="i" title="t" href="/">"#, &config).unwrap(),
            r#"<a href="/" title="t" id="i">"#
        );
    }
}
