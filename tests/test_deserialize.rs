#[cfg(test)]
mod test {
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

    #[test]
    fn deserialize_vec() {
        #[derive(Debug, Deserialize, Eq, PartialEq)]
        struct MyStruct {
            a: Vec<String>,
            b: Option<Vec<String>>,
            c: Vec<i32>,
        }

        let expected = MyStruct {
            a: vec![
                String::from("foo"),
                String::from("bar"),
                String::from("baz"),
            ],
            b: Some(vec![String::from("foo")]),
            c: vec![100, 200],
        };
        let source = "a=foo,bar,baz&b=foo&c=100,200";
        let actual: MyStruct = serde_structuredqs::from_str(source).unwrap();
        assert_eq!(actual, expected)
    }

    #[test]
    fn deserialize_vec_with_unwise_comma_separated() {
        #[derive(Debug, Deserialize, Eq, PartialEq)]
        struct MyStruct {
            a: Vec<String>,
        }

        let expected = MyStruct {
            a: vec![
                String::from("foo"),
                String::from("bar"),
                String::from("baz"),
            ],
        };
        let actual: MyStruct = serde_structuredqs::from_str("a=,,,,foo,,,,,,bar,baz,,,,").unwrap();
        assert_eq!(actual, expected)
    }

    #[test]
    fn deserialize_hollow_vec() {
        #[derive(Debug, Deserialize, Eq, PartialEq)]
        struct MyStruct {
            a: Vec<String>,
            b: Option<Vec<String>>,
        }

        let expected = MyStruct {
            a: vec![],
            b: Some(vec![]),
        };
        let actual: MyStruct = serde_structuredqs::from_str("a=,,,,,&b=,,,,").unwrap();
        assert_eq!(actual, expected)
    }

    #[test]
    fn deserialize_empty_vec() {
        #[derive(Debug, Deserialize, Eq, PartialEq)]
        struct MyStruct {
            a: Vec<String>,
            b: Option<Vec<String>>,
        }

        let expected = MyStruct { a: vec![], b: None };
        let actual: MyStruct = serde_structuredqs::from_str("a=&b=").unwrap();
        assert_eq!(actual, expected)
    }
}
