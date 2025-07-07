#!/usr/bin/env python3
"""
Cross-platform test result comparison tool

Compares test results from different binding targets (WASM, Python, FFI)
to ensure feature parity and consistent behavior.
"""

import json
import sys
from pathlib import Path
from typing import Dict, List, Any, Optional
from dataclasses import dataclass

project_root = Path(__file__).parent.parent.parent


@dataclass
class ComparisonResult:
    """Result of comparing tests across platforms"""
    platforms_tested: List[str]
    total_tests: int
    consistent_passes: int
    consistent_failures: int
    inconsistent_results: List[Dict[str, Any]]
    performance_comparison: Dict[str, Dict[str, float]]
    success_rate_by_platform: Dict[str, float]
    overall_consistency: float


class TestResultComparator:
    """Compares test results across different binding platforms"""
    
    def __init__(self):
        self.results = {}
        
    def load_results(self, platform: str, file_path: Path) -> bool:
        """Load test results for a platform"""
        try:
            if not file_path.exists():
                print(f"⚠️  Warning: Results file not found for {platform}: {file_path}")
                return False
                
            with open(file_path, 'r') as f:
                data = json.load(f)
                self.results[platform] = data
                print(f"✅ Loaded {platform} results: {data['summary']['passed']}/{data['summary']['total']} passed")
                return True
        except Exception as e:
            print(f"❌ Failed to load {platform} results: {e}")
            return False
    
    def compare_all(self) -> ComparisonResult:
        """Compare results across all loaded platforms"""
        if len(self.results) < 2:
            raise ValueError("Need at least 2 platforms to compare")
        
        platforms = list(self.results.keys())
        print(f"\n🔍 Comparing results across platforms: {', '.join(platforms)}")
        
        # Get all test names that appear in any platform
        all_test_names = set()
        for platform_results in self.results.values():
            all_test_names.update(platform_results['results'].keys())
        
        all_test_names = sorted(all_test_names)
        
        consistent_passes = 0
        consistent_failures = 0
        inconsistent_results = []
        
        for test_name in all_test_names:
            test_results = {}
            
            # Collect results for this test across all platforms
            for platform in platforms:
                if test_name in self.results[platform]['results']:
                    test_results[platform] = self.results[platform]['results'][test_name]
                else:
                    test_results[platform] = {'passed': False, 'error_message': 'Test not found'}
            
            # Check consistency
            pass_states = [result['passed'] for result in test_results.values()]
            
            if all(pass_states):
                consistent_passes += 1
                print(f"✅ {test_name}: All platforms PASS")
            elif not any(pass_states):
                consistent_failures += 1
                print(f"❌ {test_name}: All platforms FAIL")
            else:
                inconsistent_results.append({
                    'test_name': test_name,
                    'results': test_results
                })
                print(f"⚠️  {test_name}: INCONSISTENT results")
                for platform, result in test_results.items():
                    status = "PASS" if result['passed'] else f"FAIL ({result.get('error_message', 'Unknown error')})"
                    print(f"   {platform}: {status}")
        
        # Performance comparison
        performance_comparison = self._compare_performance(all_test_names)
        
        # Success rates by platform
        success_rates = {}
        for platform in platforms:
            summary = self.results[platform]['summary']
            success_rates[platform] = summary['success_rate']
        
        # Calculate overall consistency
        total_comparisons = len(all_test_names)
        consistent_results = consistent_passes + consistent_failures
        overall_consistency = consistent_results / total_comparisons if total_comparisons > 0 else 0
        
        return ComparisonResult(
            platforms_tested=platforms,
            total_tests=len(all_test_names),
            consistent_passes=consistent_passes,
            consistent_failures=consistent_failures,
            inconsistent_results=inconsistent_results,
            performance_comparison=performance_comparison,
            success_rate_by_platform=success_rates,
            overall_consistency=overall_consistency
        )
    
    def _compare_performance(self, test_names: List[str]) -> Dict[str, Dict[str, float]]:
        """Compare performance metrics across platforms"""
        performance = {}
        
        for test_name in test_names:
            test_perf = {}
            
            for platform, platform_results in self.results.items():
                if test_name in platform_results['results']:
                    result = platform_results['results'][test_name]
                    if result.get('metrics') and result['metrics'].get('execution_time_ms') is not None:
                        test_perf[platform] = result['metrics']['execution_time_ms']
            
            if len(test_perf) > 1:  # Only include if we have multiple platforms to compare
                performance[test_name] = test_perf
        
        return performance
    
    def generate_report(self, comparison: ComparisonResult) -> str:
        """Generate a detailed comparison report"""
        report = []
        report.append("=" * 80)
        report.append("🔬 CROSS-PLATFORM BINDING TEST COMPARISON REPORT")
        report.append("=" * 80)
        report.append("")
        
        # Summary
        report.append("📊 SUMMARY")
        report.append("-" * 40)
        report.append(f"Platforms tested: {', '.join(comparison.platforms_tested)}")
        report.append(f"Total tests: {comparison.total_tests}")
        report.append(f"Consistent passes: {comparison.consistent_passes}")
        report.append(f"Consistent failures: {comparison.consistent_failures}")
        report.append(f"Inconsistent results: {len(comparison.inconsistent_results)}")
        report.append(f"Overall consistency: {comparison.overall_consistency:.1%}")
        report.append("")
        
        # Success rates
        report.append("📈 SUCCESS RATES BY PLATFORM")
        report.append("-" * 40)
        for platform, rate in comparison.success_rate_by_platform.items():
            report.append(f"{platform:12}: {rate:.1%}")
        report.append("")
        
        # Inconsistent results details
        if comparison.inconsistent_results:
            report.append("⚠️  INCONSISTENT RESULTS")
            report.append("-" * 40)
            for inconsistency in comparison.inconsistent_results:
                report.append(f"\n{inconsistency['test_name']}:")
                for platform, result in inconsistency['results'].items():
                    status = "PASS" if result['passed'] else f"FAIL: {result.get('error_message', 'Unknown')}"
                    report.append(f"  {platform:12}: {status}")
            report.append("")
        
        # Performance comparison
        if comparison.performance_comparison:
            report.append("⚡ PERFORMANCE COMPARISON (execution time in ms)")
            report.append("-" * 40)
            for test_name, timings in comparison.performance_comparison.items():
                report.append(f"\n{test_name}:")
                for platform, time_ms in timings.items():
                    report.append(f"  {platform:12}: {time_ms:8.2f}ms")
                
                if len(timings) > 1:
                    fastest = min(timings.values())
                    slowest = max(timings.values())
                    if fastest > 0:
                        speedup = slowest / fastest
                        report.append(f"  {'speedup':12}: {speedup:8.2f}x")
            report.append("")
        
        # Recommendations
        report.append("💡 RECOMMENDATIONS")
        report.append("-" * 40)
        
        if comparison.overall_consistency < 0.8:
            report.append("❌ LOW CONSISTENCY: Significant differences between platforms detected!")
            report.append("   - Review inconsistent test results")
            report.append("   - Check for platform-specific implementation differences")
            report.append("   - Verify API definition completeness")
        elif comparison.overall_consistency < 0.95:
            report.append("⚠️  MODERATE CONSISTENCY: Some differences detected")
            report.append("   - Review failing tests across platforms")
            report.append("   - Consider improving error handling")
        else:
            report.append("✅ HIGH CONSISTENCY: Bindings are working well across platforms!")
            
        if len(comparison.platforms_tested) < 3:
            report.append("📝 Consider testing additional platforms (FFI, other targets)")
            
        report.append("")
        report.append("=" * 80)
        
        return "\n".join(report)


def main():
    """Main comparison function"""
    print("🔬 Cross-platform binding test comparison")
    print("=" * 50)
    
    comparator = TestResultComparator()
    
    # Look for result files
    result_files = {
        'wasm': project_root / 'test-results-wasm.json',
        'python': project_root / 'test-results-python.json',
        'ffi': project_root / 'test-results-ffi.json',  # Future
    }
    
    loaded_platforms = 0
    for platform, file_path in result_files.items():
        if comparator.load_results(platform, file_path):
            loaded_platforms += 1
    
    if loaded_platforms < 2:
        print(f"\n❌ Error: Need at least 2 platforms to compare, found {loaded_platforms}")
        print("Run tests first:")
        print("  JavaScript/WASM: cd tests/bindings/js && npm test")
        print("  Python: python tests/bindings/python/test_runner.py")
        return False
    
    try:
        comparison = comparator.compare_all()
        report = comparator.generate_report(comparison)
        
        print(report)
        
        # Save report to file
        report_path = project_root / 'binding-comparison-report.txt'
        with open(report_path, 'w') as f:
            f.write(report)
        print(f"📁 Full report saved to: {report_path}")
        
        # Return success if consistency is good
        return comparison.overall_consistency >= 0.8
        
    except Exception as e:
        print(f"❌ Comparison failed: {e}")
        return False


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
