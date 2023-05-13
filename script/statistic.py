import numpy as np
import sys

def main():
    result_file = sys.argv[1]
    test_count = int(sys.argv[2])

    with open(result_file, 'r') as f:
        content = f.read()
    result_list = [x for x in content.split(" ")]
    result_list.pop(-1)
    result = np.array([float(x) for x in result_list])
    mean = np.mean(result)
    std = np.std(result)

    print("avg throughput: ", result.size / test_count)
    print("avg delay: ", mean)
    print("delay std: ", std)

if __name__ == "__main__":
    main()