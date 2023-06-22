import pandas as pd
from pandas.core.frame import DataFrame
import seaborn as sns
import matplotlib.pyplot as plt

def get_array(file_path):
    f = open(file_path, "r",encoding='utf-8')
    line = f.readline().split(" ")
    line.pop()
    array = [float(i) for i in line]
    # array.sort()
    # print(array)
    array = [x for x in array if x < 50000]
    array = DataFrame({"delay": array})
    return array

server_use_prio = 8


# colors = ["pink", "skyblue", "red", "green", "black", "purple", "yellow", "orange"]

# for i in range(server_use_prio):
#     dir = "./test_with_prio/"
#     filename = "prio_" + str(i)
#     array = get_array(dir + filename)
#     sns.kdeplot(array, color="purple", label=filename, shade=True)

connect_with_prio_0 = get_array("./test_with_prio/prio_0")
connect_with_prio_1 = get_array("./test_with_prio/prio_1")
connect_with_prio_2 = get_array("./test_with_prio/prio_2")
connect_with_prio_3 = get_array("./test_with_prio/prio_3")
connect_with_prio_4 = get_array("./test_with_prio/prio_4")
connect_with_prio_5 = get_array("./test_with_prio/prio_5")
connect_with_prio_6 = get_array("./test_with_prio/prio_6")
connect_with_prio_7 = get_array("./test_with_prio/prio_7")


sns.kdeplot(connect_with_prio_0["delay"], color="pink", label="connect_with_prio_0", shade=True, bw_adjust=0.5)
sns.kdeplot(connect_with_prio_1["delay"], color="skyblue", label="connect_with_prio_1", shade=True, bw_adjust=0.5)
sns.kdeplot(connect_with_prio_2["delay"], color="red", label="connect_with_prio_2", shade=True, bw_adjust=0.5)
sns.kdeplot(connect_with_prio_3["delay"], color="green", label="connect_with_prio_3", shade=True, bw_adjust=0.5)
sns.kdeplot(connect_with_prio_4["delay"], color="black", label="connect_with_prio_4", shade=True, bw_adjust=0.5)
sns.kdeplot(connect_with_prio_5["delay"], color="purple", label="connect_with_prio_5", shade=True, bw_adjust=0.5)
sns.kdeplot(connect_with_prio_6["delay"], color="yellow", label="connect_with_prio_6", shade=True, bw_adjust=0.5)
sns.kdeplot(connect_with_prio_7["delay"], color="orange", label="connect_with_prio_7", shade=True, bw_adjust=0.5)
plt.legend()
plt.savefig("./connect_with_prio_test.png")