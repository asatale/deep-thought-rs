/// Example demonstrating range queries and iteration patterns
///
/// Run with: cargo run --example range_queries

use skiplist::{SkipList, SkipListEntry, SkipListNode};

#[derive(Debug)]
struct Product {
    price: u64, // Price in cents
    name: String,
    category: String,
    in_stock: bool,
    skiplist_meta: SkipListNode,
}

impl Product {
    fn new(price: u64, name: &str, category: &str, in_stock: bool) -> Box<Self> {
        Box::new(Product {
            price,
            name: name.to_string(),
            category: category.to_string(),
            in_stock,
            skiplist_meta: SkipListNode::new(),
        })
    }

    fn price_display(&self) -> String {
        format!("${}.{:02}", self.price / 100, self.price % 100)
    }
}

impl SkipListEntry for Product {
    type Key = u64;
    fn key(&self) -> &Self::Key {
        &self.price
    }
    fn skiplist_node(&self) -> &SkipListNode {
        &self.skiplist_meta
    }
    fn skiplist_node_mut(&mut self) -> &mut SkipListNode {
        &mut self.skiplist_meta
    }
}

fn main() {
    println!("=== Skip List: Range Queries and Iteration ===\n");

    // Example 1: Simple range queries
    example_range_queries();

    // Example 2: Pagination
    example_pagination();

    // Example 3: Finding gaps
    example_find_gaps();

    // Example 4: Filtering while iterating
    example_filtered_iteration();

    // Example 5: Batch operations
    example_batch_operations();
}

fn create_product_catalog() -> SkipList<u64, Product> {
    let mut catalog: SkipList<u64, Product> = SkipList::new();

    // Add products with prices in cents
    catalog.insert(Product::new(999, "Coffee Mug", "Kitchen", true)).unwrap();
    catalog.insert(Product::new(2499, "Wireless Mouse", "Electronics", true)).unwrap();
    catalog.insert(Product::new(4999, "Bluetooth Speaker", "Electronics", false)).unwrap();
    catalog.insert(Product::new(1499, "Notebook", "Office", true)).unwrap();
    catalog.insert(Product::new(899, "Pen Set", "Office", true)).unwrap();
    catalog.insert(Product::new(7999, "Mechanical Keyboard", "Electronics", true)).unwrap();
    catalog.insert(Product::new(3499, "Desk Lamp", "Office", true)).unwrap();
    catalog.insert(Product::new(1999, "Water Bottle", "Kitchen", true)).unwrap();
    catalog.insert(Product::new(5999, "Backpack", "Accessories", false)).unwrap();
    catalog.insert(Product::new(12999, "Monitor", "Electronics", true)).unwrap();

    catalog
}

fn example_range_queries() {
    println!("Example 1: Range Queries");
    println!("------------------------");

    let catalog = create_product_catalog();

    println!("Total products: {}\n", catalog.len());

    // Query: Find products in price range [2000, 5000] cents ($20-$50)
    println!("Products between $20 and $50:");
    let min_price = 2000;
    let max_price = 5000;

    let mut current = catalog.get(&min_price).or_else(|| catalog.successor(&min_price));
    let mut count = 0;

    while let Some(product) = current {
        if *product.key() > max_price {
            break;
        }
        println!(
            "  {} - {} ({})",
            product.price_display(),
            product.name,
            product.category
        );
        count += 1;
        current = catalog.successor(product.key());
    }
    println!("Found {} products\n", count);

    // Query: Find cheapest product
    if let Some(cheapest) = catalog.first() {
        println!(
            "Cheapest product: {} ({})",
            cheapest.name,
            cheapest.price_display()
        );
    }

    // Query: Find products above $100
    let expensive_threshold = 10000;
    if let Some(first_expensive) = catalog.successor(&expensive_threshold) {
        println!(
            "First product above $100: {} ({})",
            first_expensive.name,
            first_expensive.price_display()
        );
    }

    println!();
}

fn example_pagination() {
    println!("Example 2: Pagination (Page-by-Page Results)");
    println!("---------------------------------------------");

    let catalog = create_product_catalog();

    let page_size = 3;
    let mut page_num = 1;
    let mut current = catalog.first();

    println!("Total products: {}", catalog.len());
    println!("Page size: {}\n", page_size);

    while current.is_some() {
        println!("Page {}:", page_num);

        let mut items_in_page = 0;
        let mut last_key = None;

        while let Some(product) = current {
            println!(
                "  {} - {} ({})",
                product.price_display(),
                product.name,
                if product.in_stock {
                    "In Stock"
                } else {
                    "Out of Stock"
                }
            );

            last_key = Some(*product.key());
            items_in_page += 1;

            if items_in_page >= page_size {
                break;
            }

            current = catalog.successor(product.key());
        }

        // Move to next page
        if let Some(key) = last_key {
            current = catalog.successor(&key);
        } else {
            break;
        }

        if current.is_some() {
            println!();
        }
        page_num += 1;
    }

    println!();
}

fn example_find_gaps() {
    println!("Example 3: Finding Gaps in Price Points");
    println!("----------------------------------------");

    let catalog = create_product_catalog();

    println!("Analyzing price gaps...\n");

    let mut current = catalog.first();
    let mut prev_price = None;
    let mut max_gap = 0u64;
    let mut max_gap_range = (0, 0);

    while let Some(product) = current {
        if let Some(prev) = prev_price {
            let gap = product.price - prev;
            if gap > max_gap {
                max_gap = gap;
                max_gap_range = (prev, product.price);
            }

            if gap > 1000 {
                // Gaps larger than $10
                println!(
                    "  Large gap: ${}.{:02} -> ${}.{:02} (gap: ${}.{:02})",
                    prev / 100,
                    prev % 100,
                    product.price / 100,
                    product.price % 100,
                    gap / 100,
                    gap % 100
                );
            }
        }

        prev_price = Some(product.price);
        current = catalog.successor(product.key());
    }

    println!(
        "\nLargest gap: ${}.{:02} -> ${}.{:02} (${}.{:02})",
        max_gap_range.0 / 100,
        max_gap_range.0 % 100,
        max_gap_range.1 / 100,
        max_gap_range.1 % 100,
        max_gap / 100,
        max_gap % 100
    );

    println!();
}

fn example_filtered_iteration() {
    println!("Example 4: Filtered Iteration");
    println!("------------------------------");

    let catalog = create_product_catalog();

    // Filter 1: In-stock items only
    println!("In-stock products:");
    let mut current = catalog.first();
    let mut count = 0;

    while let Some(product) = current {
        if product.in_stock {
            println!(
                "  {} - {} ({})",
                product.price_display(),
                product.name,
                product.category
            );
            count += 1;
        }
        current = catalog.successor(product.key());
    }
    println!("Total in-stock: {}\n", count);

    // Filter 2: Electronics category only, under $80
    println!("Electronics under $80:");
    let max_price = 8000;
    let mut current = catalog.first();
    let mut count = 0;

    while let Some(product) = current {
        if *product.key() > max_price {
            break;
        }

        if product.category == "Electronics" {
            println!(
                "  {} - {} ({})",
                product.price_display(),
                product.name,
                if product.in_stock { "✓" } else { "✗" }
            );
            count += 1;
        }

        current = catalog.successor(product.key());
    }
    println!("Found {} electronics\n", count);
}

fn example_batch_operations() {
    println!("Example 5: Batch Operations");
    println!("----------------------------");

    let mut catalog = create_product_catalog();

    println!("Initial count: {}\n", catalog.len());

    // Batch operation: Remove all items under $15
    println!("Removing items under $15...");
    let threshold = 1500;
    let mut to_remove = Vec::new();

    let mut current = catalog.first();
    while let Some(product) = current {
        if *product.key() >= threshold {
            break;
        }
        println!("  Marking for removal: {} ({})", product.name, product.price_display());
        to_remove.push(*product.key());
        current = catalog.successor(product.key());
    }

    // Remove collected items
    for key in to_remove {
        catalog.remove(&key);
    }
    println!("Removed {} items", catalog.len());

    // Batch operation: Update prices (discount on expensive items)
    println!("\nApplying 10% discount on items over $50...");
    let discount_threshold = 5000;
    let mut updates = Vec::new();

    let mut current = catalog.successor(&discount_threshold);
    while let Some(product) = current {
        let new_price = (product.price as f64 * 0.9) as u64;
        println!(
            "  {} - {} -> {}",
            product.name,
            product.price_display(),
            format!("${}.{:02}", new_price / 100, new_price % 100)
        );

        // Collect updates (need to remove and re-insert with new key)
        updates.push((
            *product.key(),
            product.name.clone(),
            product.category.clone(),
            product.in_stock,
            new_price,
        ));

        current = catalog.successor(product.key());
    }

    // Apply updates
    for (old_price, name, category, in_stock, new_price) in updates {
        catalog.remove(&old_price);
        catalog.insert(Product::new(new_price, &name, &category, in_stock)).unwrap();
    }

    println!("\nFinal count: {}", catalog.len());
    println!("\nUpdated catalog (sorted by price):");
    let mut current = catalog.first();
    while let Some(product) = current {
        println!(
            "  {} - {} ({})",
            product.price_display(),
            product.name,
            product.category
        );
        current = catalog.successor(product.key());
    }

    println!();
}
