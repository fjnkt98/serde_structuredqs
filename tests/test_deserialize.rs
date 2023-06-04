use serde::Deserialize;

#[test]
fn test_deserialize_flat() {
    #[derive(Debug, Deserialize, Eq, PartialEq)]
    struct SearchParams {
        keyword: Option<String>,
        limit: Option<u32>,
    }

    let expected = SearchParams {
        keyword: Some(String::from("foo")),
        limit: Some(20),
    };

    let source = "keyword=foo&limit=20";
    let actual: SearchParams = serde_structuredqs::from_str(source).unwrap();
    assert_eq!(actual, expected)
}

#[test]
fn test_deserialize_structured() {
    #[derive(Debug, Deserialize, Eq, PartialEq)]
    struct SearchParams {
        keyword: Option<String>,
        limit: Option<u32>,
        filter: Option<FilteringParameter>,
    }

    #[derive(Debug, Deserialize, Eq, PartialEq)]
    struct FilteringParameter {
        category: Option<String>,
        difficulty: Option<RangeFilteringParameter<i32>>,
    }

    #[derive(Debug, Deserialize, Eq, PartialEq)]
    struct RangeFilteringParameter<T> {
        from: Option<T>,
        to: Option<T>,
    }

    let expected = SearchParams {
        keyword: Some(String::from("foo")),
        limit: Some(20),
        filter: Some(FilteringParameter {
            category: Some(String::from("A")),
            difficulty: Some(RangeFilteringParameter {
                from: None,
                to: Some(800),
            }),
        }),
    };

    let source = "keyword=foo&limit=20&filter.category=A&filter.difficulty.to=800";
    let actual: SearchParams = serde_structuredqs::from_str(source).unwrap();
    assert_eq!(actual, expected)
}
