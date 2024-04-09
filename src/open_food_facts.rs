pub mod models;
pub mod sdk;

//tests
#[cfg(test)]
mod tests {
    use super::sdk::search_openff;
    use super::models::{ OpenFoodFactsQuery};

    #[tokio::test]
    async fn test_search_nutella() {
        let query = OpenFoodFactsQuery {
            search_query: "Nutella".to_string(),
            tags: vec!["de".to_string()],
        };
        match search_openff(query).await {
            Ok(result) => {
                for product in result.products {
                    println!("Product code: {:?}", product.code);
                    println!("Product name: {:?}", product.product_name);
                    println!("Nutrition grade: {:?}", product.nutrition_grades);
                    println!("Nutrients: {:?}", product.nutriments);
                    println!("Energy: {:?} ", product.nutriments.clone().unwrap().energy_kcal_100g, );
                    println!("----------------------");
                }
            }
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }
    #[tokio::test]
    async fn test_search_pizza() {
        let query = OpenFoodFactsQuery {
            search_query: "pizza".to_string(),
            tags: vec!["pizza".to_string()],
        };
        let result = search_openff(query).await;
        assert!(result.is_ok());
    }

}