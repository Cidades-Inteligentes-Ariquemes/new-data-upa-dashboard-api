def load_file_to_dictionary(file_path):
    result_dict = {}

    with open(file_path, 'r') as file:
        for line in file:
            value, disease = line.strip().split(' ', 1)
            result_dict[disease] = float(value) * 100  
    return result_dict