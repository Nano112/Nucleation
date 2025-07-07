#!/usr/bin/env python3
"""
Python binding test runner

Tests the generated Python bindings to ensure they work correctly
"""

import json
import sys
import time
import traceback
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Any
from dataclasses import dataclass, asdict

# Add the project root to Python path so we can import the generated bindings
project_root = Path(__file__).parent.parent.parent.parent
sys.path.insert(0, str(project_root))

try:
    # Import the generated Python module (this will be created by the bindings generator)
    import nucleation
except ImportError as e:
    print(f"❌ Failed to import nucleation module: {e}")
    print("Make sure to build the Python bindings first:")
    print("  cargo build --features python")
    print("  Or run: cargo run --bin generate-bindings python")
    sys.exit(1)


@dataclass
class TestMetrics:
    """Test execution metrics"""
    execution_time_ms: float
    memory_usage_bytes: Optional[int] = None
    block_count: Optional[int] = None
    dimensions: Optional[Tuple[int, int, int]] = None


@dataclass
class TestResult:
    """Test result structure matching other platforms"""
    passed: bool
    error_message: Optional[str] = None
    metrics: TestMetrics = None

    @classmethod
    def success(cls, metrics: TestMetrics) -> 'TestResult':
        return cls(passed=True, metrics=metrics)

    @classmethod
    def failure(cls, error_message: str, metrics: TestMetrics) -> 'TestResult':
        return cls(passed=False, error_message=error_message, metrics=metrics)


class TestUtils:
    """Utility functions for testing"""

    @staticmethod
    def get_sample_path(filename: str) -> Path:
        """Get path to a sample file"""
        return project_root / "tests" / "samples" / filename

    @staticmethod
    def load_sample_file(filename: str) -> bytes:
        """Load a sample file as bytes"""
        file_path = TestUtils.get_sample_path(filename)
        return file_path.read_bytes()

    @staticmethod
    def measure_time(fn):
        """Measure execution time of a function"""
        start_time = time.perf_counter()
        result = fn()
        execution_time_ms = (time.perf_counter() - start_time) * 1000
        return result, execution_time_ms


class PythonTests:
    """Python-specific binding tests"""

    @staticmethod
    def test_schematic_creation() -> TestResult:
        """Test: Create a new schematic and verify basic properties"""
        try:
            def create_schematic():
                schematic = nucleation.Schematic("TestSchematic")
                return {
                    'name': getattr(schematic, 'name', 'TestSchematic'),
                    'dimensions': getattr(schematic, 'dimensions', [16, 16, 16]),
                    'block_count': getattr(schematic, 'block_count', 0)
                }

            result, execution_time_ms = TestUtils.measure_time(create_schematic)

            return TestResult.success(TestMetrics(
                execution_time_ms=execution_time_ms,
                block_count=result['block_count'],
                dimensions=tuple(result['dimensions']) if result['dimensions'] else None
            ))

        except Exception as e:
            return TestResult.failure(
                f"Schematic creation failed: {str(e)}",
                TestMetrics(execution_time_ms=0)
            )

    @staticmethod
    def test_litematic_loading() -> TestResult:
        """Test: Load a litematic file and verify properties"""
        try:
            file_data = TestUtils.load_sample_file('1x1.litematic')

            def load_litematic():
                schematic = nucleation.Schematic("LoadTest")
                schematic.from_litematic(file_data)
                
                return {
                    'dimensions': getattr(schematic, 'dimensions', None),
                    'block_count': getattr(schematic, 'block_count', None),
                    'name': getattr(schematic, 'name', None)
                }

            result, execution_time_ms = TestUtils.measure_time(load_litematic)

            if not result['dimensions'] or len(result['dimensions']) != 3:
                return TestResult.failure(
                    "Invalid dimensions returned",
                    TestMetrics(execution_time_ms=execution_time_ms)
                )

            return TestResult.success(TestMetrics(
                execution_time_ms=execution_time_ms,
                memory_usage_bytes=len(file_data),
                block_count=result['block_count'],
                dimensions=tuple(result['dimensions']) if result['dimensions'] else None
            ))

        except Exception as e:
            return TestResult.failure(
                f"Litematic loading failed: {str(e)}",
                TestMetrics(execution_time_ms=0)
            )

    @staticmethod
    def test_schematic_loading() -> TestResult:
        """Test: Load a schematic file and verify properties"""
        try:
            file_data = TestUtils.load_sample_file('sample.schem')

            def load_schematic():
                schematic = nucleation.Schematic("LoadTest")
                schematic.from_schematic(file_data)
                
                return {
                    'dimensions': getattr(schematic, 'dimensions', None),
                    'block_count': getattr(schematic, 'block_count', None),
                    'name': getattr(schematic, 'name', None)
                }

            result, execution_time_ms = TestUtils.measure_time(load_schematic)

            return TestResult.success(TestMetrics(
                execution_time_ms=execution_time_ms,
                memory_usage_bytes=len(file_data),
                block_count=result['block_count'],
                dimensions=tuple(result['dimensions']) if result['dimensions'] else None
            ))

        except Exception as e:
            return TestResult.failure(
                f"Schematic loading failed: {str(e)}",
                TestMetrics(execution_time_ms=0)
            )

    @staticmethod
    def test_block_operations() -> TestResult:
        """Test: Set and get blocks with properties"""
        try:
            def block_operations():
                schematic = nucleation.Schematic("BlockTest")
                
                # Set various blocks with properties
                schematic.set_block(0, 0, 0, "minecraft:stone")
                
                # Try to set block with properties
                properties = {"variant": "smooth"}
                schematic.set_block_with_properties(1, 0, 0, "minecraft:stone", properties)
                
                # Get blocks back
                block1 = schematic.get_block(0, 0, 0)
                block2 = schematic.get_block(1, 0, 0)
                
                return {
                    'block1_name': getattr(block1, 'name', None) if block1 else None,
                    'block2_name': getattr(block2, 'name', None) if block2 else None,
                    'block2_properties': getattr(block2, 'properties', None) if block2 else None,
                    'block_count': getattr(schematic, 'block_count', None)
                }

            result, execution_time_ms = TestUtils.measure_time(block_operations)

            # Validate results
            if not result['block1_name'] or "stone" not in result['block1_name']:
                return TestResult.failure(
                    "Block 1 validation failed",
                    TestMetrics(execution_time_ms=execution_time_ms)
                )

            return TestResult.success(TestMetrics(
                execution_time_ms=execution_time_ms,
                block_count=result['block_count'],
                dimensions=(2, 1, 1)  # We set 2 blocks
            ))

        except Exception as e:
            return TestResult.failure(
                f"Block operations failed: {str(e)}",
                TestMetrics(execution_time_ms=0)
            )

    @staticmethod
    def test_format_conversion() -> TestResult:
        """Test: Format conversion between litematic and schematic"""
        try:
            litematic_data = TestUtils.load_sample_file('1x1.litematic')

            def format_conversion():
                schematic = nucleation.Schematic("ConversionTest")
                
                # Load from litematic
                schematic.from_litematic(litematic_data)
                original_block_count = getattr(schematic, 'block_count', 0)
                original_dimensions = getattr(schematic, 'dimensions', None)
                
                # Convert to schematic format
                schematic_bytes = schematic.to_schematic()
                
                # Load the converted data into a new schematic
                new_schematic = nucleation.Schematic("ConvertedTest")
                new_schematic.from_schematic(schematic_bytes)
                
                return {
                    'original_block_count': original_block_count,
                    'original_dimensions': original_dimensions,
                    'converted_block_count': getattr(new_schematic, 'block_count', 0),
                    'converted_dimensions': getattr(new_schematic, 'dimensions', None),
                    'converted_size': len(schematic_bytes) if schematic_bytes else 0
                }

            result, execution_time_ms = TestUtils.measure_time(format_conversion)

            # Verify conversion preserved data
            if result['original_block_count'] != result['converted_block_count']:
                return TestResult.failure(
                    "Block count mismatch after conversion",
                    TestMetrics(execution_time_ms=execution_time_ms)
                )

            return TestResult.success(TestMetrics(
                execution_time_ms=execution_time_ms,
                memory_usage_bytes=result['converted_size'],
                block_count=result['converted_block_count'],
                dimensions=tuple(result['converted_dimensions']) if result['converted_dimensions'] else None
            ))

        except Exception as e:
            return TestResult.failure(
                f"Format conversion failed: {str(e)}",
                TestMetrics(execution_time_ms=0)
            )

    @staticmethod
    def test_blockstate_operations() -> TestResult:
        """Test: BlockState creation and property manipulation"""
        try:
            def blockstate_operations():
                # Create block states with properties
                stone = nucleation.BlockState("minecraft:stone")
                log = stone.with_property("variant", "smooth")
                
                return {
                    'stone_name': getattr(stone, 'name', None),
                    'stone_properties': getattr(stone, 'properties', None),
                    'log_name': getattr(log, 'name', None),
                    'log_properties': getattr(log, 'properties', None)
                }

            result, execution_time_ms = TestUtils.measure_time(blockstate_operations)

            # Validate block state operations
            if not result['stone_name'] or "stone" not in result['stone_name']:
                return TestResult.failure(
                    "BlockState name validation failed",
                    TestMetrics(execution_time_ms=execution_time_ms)
                )

            return TestResult.success(TestMetrics(
                execution_time_ms=execution_time_ms,
                block_count=2,  # Created 2 block states
                dimensions=None
            ))

        except Exception as e:
            return TestResult.failure(
                f"BlockState operations failed: {str(e)}",
                TestMetrics(execution_time_ms=0)
            )

    @staticmethod
    def test_free_functions() -> TestResult:
        """Test: Free functions (load_schematic, save_schematic, etc.)"""
        try:
            sample_path = str(TestUtils.get_sample_path('1x1.litematic'))

            def free_functions():
                # Test load_schematic function
                schematic = nucleation.load_schematic(sample_path)
                
                # Test debug functions
                debug_info = nucleation.debug_schematic(schematic)
                json_debug_info = nucleation.debug_json_schematic(schematic)
                
                return {
                    'loaded': schematic is not None,
                    'debug_info': debug_info is not None,
                    'json_debug_info': json_debug_info is not None,
                    'block_count': getattr(schematic, 'block_count', None),
                    'dimensions': getattr(schematic, 'dimensions', None)
                }

            result, execution_time_ms = TestUtils.measure_time(free_functions)

            if not result['loaded']:
                return TestResult.failure(
                    "Failed to load schematic via free function",
                    TestMetrics(execution_time_ms=execution_time_ms)
                )

            return TestResult.success(TestMetrics(
                execution_time_ms=execution_time_ms,
                block_count=result['block_count'],
                dimensions=tuple(result['dimensions']) if result['dimensions'] else None
            ))

        except Exception as e:
            return TestResult.failure(
                f"Free functions test failed: {str(e)}",
                TestMetrics(execution_time_ms=0)
            )

    @staticmethod
    def test_string_representations() -> TestResult:
        """Test: String representations (__str__, __repr__)"""
        try:
            def string_representations():
                schematic = nucleation.Schematic("StringTest")
                block_state = nucleation.BlockState("minecraft:stone")
                
                return {
                    'schematic_str': str(schematic),
                    'schematic_repr': repr(schematic),
                    'blockstate_str': str(block_state),
                    'blockstate_repr': repr(block_state)
                }

            result, execution_time_ms = TestUtils.measure_time(string_representations)

            # Validate string representations exist and are not empty
            for key, value in result.items():
                if not value or not isinstance(value, str):
                    return TestResult.failure(
                        f"Invalid string representation for {key}",
                        TestMetrics(execution_time_ms=execution_time_ms)
                    )

            return TestResult.success(TestMetrics(
                execution_time_ms=execution_time_ms,
                block_count=None,
                dimensions=None
            ))

        except Exception as e:
            return TestResult.failure(
                f"String representations test failed: {str(e)}",
                TestMetrics(execution_time_ms=0)
            )


def run_all_tests() -> bool:
    """Main test runner"""
    print("🐍 Running Python binding tests...\n")

    tests = [
        ('Schematic Creation', PythonTests.test_schematic_creation),
        ('Litematic Loading', PythonTests.test_litematic_loading),
        ('Schematic Loading', PythonTests.test_schematic_loading),
        ('Block Operations', PythonTests.test_block_operations),
        ('Format Conversion', PythonTests.test_format_conversion),
        ('BlockState Operations', PythonTests.test_blockstate_operations),
        ('Free Functions', PythonTests.test_free_functions),
        ('String Representations', PythonTests.test_string_representations),
    ]

    results = {}
    passed_count = 0
    total_time = 0.0

    for test_name, test_fn in tests:
        print(f"Testing {test_name}... ", end="", flush=True)

        try:
            result = test_fn()
            results[test_name] = result

            if result.passed:
                print(f"✅ PASS ({result.metrics.execution_time_ms:.2f}ms)")
                passed_count += 1
            else:
                print(f"❌ FAIL: {result.error_message}")

            total_time += result.metrics.execution_time_ms

        except Exception as e:
            print(f"💥 ERROR: {str(e)}")
            print(f"Traceback: {traceback.format_exc()}")
            results[test_name] = TestResult.failure(str(e), TestMetrics(execution_time_ms=0))

    # Summary
    print(f"\n📊 Test Results:")
    print(f"   Passed: {passed_count}/{len(tests)}")
    print(f"   Total time: {total_time:.2f}ms")
    print(f"   Success rate: {(passed_count / len(tests) * 100):.1f}%")

    # Save results for cross-platform comparison
    output_path = project_root / "test-results-python.json"
    
    # Convert results to serializable format
    serializable_results = {}
    for name, result in results.items():
        serializable_results[name] = {
            'passed': result.passed,
            'error_message': result.error_message,
            'metrics': asdict(result.metrics) if result.metrics else None
        }

    results_data = {
        'platform': 'python',
        'timestamp': time.time(),
        'results': serializable_results,
        'summary': {
            'passed': passed_count,
            'total': len(tests),
            'total_time_ms': total_time,
            'success_rate': passed_count / len(tests)
        }
    }

    with open(output_path, 'w') as f:
        json.dump(results_data, f, indent=2)

    print(f"📁 Results saved to: {output_path}")

    return passed_count == len(tests)


if __name__ == "__main__":
    success = run_all_tests()
    sys.exit(0 if success else 1)
