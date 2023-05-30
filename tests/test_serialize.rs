use serde_derive::Serialize;

#[test]
fn flat_struct() {
    #[derive(Serialize)]
    struct FlatStruct {
        a: i32,
        b: f64,
        c: String,
    }

    let params = FlatStruct {
        a: 100,
        b: 3.14,
        c: String::from("foo"),
    };
    assert_eq!(
        serde_structuredqs::to_string(&params).unwrap(),
        String::from("a=100&b=3.14&c=foo")
    );
}

#[test]
fn serialize_nested_struct() {
    #[derive(Serialize)]
    struct OneNestedQueryString {
        a: i32,
        b: i32,
        c: ChildElement,
    }

    #[derive(Serialize)]
    struct ChildElement {
        d: i32,
        e: i32,
    }

    let params = OneNestedQueryString {
        a: 1,
        b: 100,
        c: ChildElement { d: 2, e: 3 },
    };
    assert_eq!(
        serde_structuredqs::to_string(&params).unwrap(),
        String::from("a=1&b=100&c.d=2&c.e=3")
    )
}

#[test]
fn serialize_search_params() {
    #[derive(Serialize)]
    struct SearchParameter {
        keyword: Option<String>,
        page: Option<u32>,
        limit: Option<u32>,
        filter: Option<FilteringParameter>,
        sort: Option<String>,
    }

    #[derive(Serialize)]
    struct FilteringParameter {
        category: Option<String>,
        difficulty: Option<RangeFilteringParameter>,
    }

    #[derive(Serialize)]
    struct RangeFilteringParameter {
        from: Option<i32>,
        to: Option<i32>,
    }

    let params = SearchParameter {
        keyword: Some(String::from("foo,bar")),
        page: None,
        limit: Some(20),
        filter: Some(FilteringParameter {
            category: Some(String::from("ABC")),
            difficulty: Some(RangeFilteringParameter {
                from: Some(800),
                to: None,
            }),
        }),
        sort: None,
    };

    assert_eq!(
        serde_structuredqs::to_string(&params).unwrap(),
        String::from("keyword=foo%2Cbar&limit=20&filter.category=ABC&filter.difficulty.from=800")
    )
}
