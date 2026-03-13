"""conda-emscripten: Emscripten-specific conda plugins (solver, extractor, virtual packages)."""

__version__ = "0.1.0"

import logging
import sys
import warnings

if sys.platform == "emscripten":
    # Emscripten can't start threads; silence tqdm's monitor warning.
    warnings.filterwarnings("ignore", message="tqdm:disabling monitor", category=FutureWarning)
    warnings.filterwarnings("ignore", message="tqdm:disabling monitor", category=UserWarning)
    try:
        from tqdm import TqdmMonitorWarning
        warnings.filterwarnings("ignore", category=TqdmMonitorWarning)
    except ImportError:
        pass

    # urllib3 DEBUG-level connection logs are noisy in the browser console.
    logging.getLogger("urllib3").setLevel(logging.WARNING)
