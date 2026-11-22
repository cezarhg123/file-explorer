pub fn string_sort(strings: &[String]) -> Vec<String> {
	let mut sorted_strings = strings.to_vec();

	if sorted_strings.len() > 1 {
		sorted_strings = sort_in_place(&mut sorted_strings);
	}

	sorted_strings
}

fn sort_in_place(list: &mut[String]) -> Vec<String> {
	if list.len() == 0 { return Vec::new(); }	

	let pivot = list[0].clone();

	let mut lower = Vec::new();
	let mut higher = Vec::new();

	if list.len() > 2 {
		// split the list into 2 lists with values lower and higher than pivot
		for i in 1..list.len() {
			if list[i] <= pivot {
				lower.push(list[i].clone());
			} else {
				higher.push(list[i].clone());
			}
		}
	} else if list.len() == 2 {
		if list[0] > list[1] {
			list.swap(0, 1);
		}

		return list.to_vec();
	}

	let mut sorted = Vec::new();
	sorted.append(&mut sort_in_place(&mut lower));
	sorted.push(pivot);
	sorted.append(&mut sort_in_place(&mut higher));

	sorted
}
