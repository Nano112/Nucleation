import nucleation
import mcschematic
import timeit

VOLUME_SIDE_LENGTH = 100
mc_schematic = mcschematic.MCSchematic()

def fill_cube_mcschematic():
    """Fill a cube with blocks in a mcschematic."""
    for x in range(VOLUME_SIDE_LENGTH):
        for y in range(VOLUME_SIDE_LENGTH):
            for z in range(VOLUME_SIDE_LENGTH):
                mc_schematic.setBlock((x, y, z), 'minecraft:stone')
    return mc_schematic

nucleation_schematic = nucleation.Schematic("test")

def fill_cube_nucleation():
    """Fill a cube with blocks using nucleation."""
    for x in range(VOLUME_SIDE_LENGTH):
        for y in range(VOLUME_SIDE_LENGTH):
            for z in range(VOLUME_SIDE_LENGTH):
                nucleation_schematic.set_block(x,y,z, 'minecraft:stone')
    return nucleation_schematic

# fill_cube_mcschematic()
# fill_cube_nucleation()

if __name__ == "__main__":
    mcschematic_time = timeit.timeit(fill_cube_mcschematic, number=1)
    nucleation_time = timeit.timeit(fill_cube_nucleation, number=1)

    print(f"MCSchematic time: {mcschematic_time:.2f} seconds")
    print(f"Nucleation time: {nucleation_time:.2f} seconds")
    print(f"Nucleation is {mcschematic_time / nucleation_time:.2f} times faster than MCSchematic.")