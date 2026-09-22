import json
import sys
from AoE2ScenarioParser.scenarios.aoe2_de_scenario import AoE2DEScenario
from AoE2ScenarioParser import settings

settings.PRINT_STATUS_UPDATES = False

path = sys.argv[1]
scenario = AoE2DEScenario.from_file(path)

result = {
    "width": scenario.map_manager.map_width,
    "height": scenario.map_manager.map_height,
}

print(json.dumps(result))