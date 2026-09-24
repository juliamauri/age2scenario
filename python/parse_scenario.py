import json
import sys
from AoE2ScenarioParser.scenarios.aoe2_de_scenario import AoE2DEScenario
from AoE2ScenarioParser import settings

settings.PRINT_STATUS_UPDATES = False

path = sys.argv[1]
scenario = AoE2DEScenario.from_file(path)

height = scenario.map_manager.map_height
width = scenario.map_manager.map_width
terrain = []
for y in range(height):
    for x in range(width):
        tile = scenario.map_manager.get_tile(x=x, y=y)
        terrain.append(tile.terrain_id)

result = {
    "width": width,
    "height": height,
    "terrain": terrain,
}

print(json.dumps(result))