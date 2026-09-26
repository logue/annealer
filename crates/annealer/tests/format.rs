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
            "<style lang=\"less\">\na { color: red; display: block; }\n</style>\n",
        );
        assert_eq!(vue(input), input);
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

mod stylesheets {
    use super::*;

    #[test]
    fn delegates_property_order_to_malva() {
        assert_eq!(
            run("a { color: red; display: block; }\n", Language::Css),
            "a {\n  display: block;\n  color: red;\n}\n"
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
