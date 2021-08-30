# ECUTelemtryData { ecu_data: ECUDataFrame { time: 0.0, igniter_state: Idle, sensor_states: [0, 0, 0, 398, 0], valve_states: [0, 0, 0, 0], sparking: false }, avg_loop_time: 0.00009406, max_loop_time: 0.000000000000000000000000000000000000000023704 }
import matplotlib.pyplot as plt
import numpy as np

config = [0.0, 300.0, 200.0, 200.0, 300.0]
names = ['n/a', 'fuel-injector', 'gox-injector', 'chamber', 'tank']

file = open('telem-data.log', 'r')

sensors = [[], [], [], [], []]
states = []
x_values = []

start = 1750
end = 3500

# Load values from file

for line in file.readlines():
    parsed = line[(line.find('[') + 1):line.find(']')]
    values = parsed.split(',')

    instant = []

    i = 0
    for val in values:
        voltage = 5.0 * (float(val) / 4095.0)
        reading = ((voltage - 0.5) / 4.0) * config[i]
        sensors[i] += [reading]
        i += 1

    s_start = line.find('igniter_state: ') + len('igniter_state: ')
    s_end = line[s_start:-1].find(',')
    states += [line[s_start:(s_start + s_end)]]

    if (len(states) >= 2 and states[-2] == 'Idle' and states[-1] == 'Prefire'):
        print('Start is ' + str(len(sensors[0]) - 500))
        start = len(sensors[0]) - 250

# Correct for 0-offset/bias
sensor_sums = [[], [], [], [], []]

for i in range(0, start - 1):
    if states[i] == 'Idle':
        for s in range(0, len(sensor_sums)):
            sensor_sums[s] += [sensors[s][i]]

sensor_avgs = [np.average(sum) for sum in sensor_sums]
sensors_to_do_avg = [0, 1, 2, 3]

for i in range(0, len(sensors[0])):
    for s in sensors_to_do_avg:
        sensors[s][i] -= sensor_avgs[s]

# Plot the lines

xpoints = np.array(np.arange(-0.25, len(sensors[0]) / 1000.0 - 0.25, 0.001))
fig, ax = plt.subplots()

for i in [1, 2, 4, 3]:
    ax.plot(xpoints[0:(end-start)], sensors[i][start:end], label=names[i])

# Color the states in the background

current_state = states[start]
state_start = 0
for i in range(1, end - start):
    if current_state != states[i + start]:
        color = 'gray'
        if states[i + start] == 'Idle':
            color = 'green'
        elif states[i + start] == 'Prefire':
            color = 'orange'
        elif states[i + start] == 'Firing':
            color = 'red'

        ax.axvspan(xpoints[state_start], xpoints[i], facecolor=color, alpha=0.25)

        state_start = i
        current_state = states[i + start]

plt.legend()
plt.show()
