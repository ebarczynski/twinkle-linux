# Contributing to Twinkle Linux

Thank you for your interest in contributing to Twinkle Linux! We welcome contributions from everyone, whether you're fixing bugs, adding features, improving documentation, or reporting issues.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Code Style Guidelines](#code-style-guidelines)
- [Commit Message Guidelines](#commit-message-guidelines)
- [Pull Request Process](#pull-request-process)
- [Testing](#testing)
- [Reporting Issues](#reporting-issues)
- [Feature Requests](#feature-requests)

## Getting Started

### Prerequisites

Before contributing, make sure you have:

- Python 3.10 or later
- Git installed and configured
- A GitHub account
- Familiarity with Python and PyQt6
- Basic understanding of DDC/CI protocol (helpful but not required)

### Setting Up Your Development Environment

1. **Fork the Repository**

   Visit the [Twinkle Linux repository](https://github.com/twinkle-linux/twinkle-linux) and click the "Fork" button in the top-right corner.

2. **Clone Your Fork**

   ```bash
   git clone https://github.com/YOUR_USERNAME/twinkle-linux.git
   cd twinkle-linux
   ```

3. **Add Upstream Remote**

   ```bash
   git remote add upstream https://github.com/twinkle-linux/twinkle-linux.git
   ```

4. **Create a Virtual Environment**

   ```bash
   python3 -m venv venv
   source venv/bin/activate  # On Windows: venv\Scripts\activate
   ```

5. **Install Dependencies**

   ```bash
   pip install -r requirements.txt
   pip install -e ".[dev]"
   ```

6. **Install System Dependencies**

   ```bash
   # Ubuntu/Debian
   sudo apt install ddcutil i2c-tools

   # Fedora/RHEL/CentOS
   sudo dnf install ddcutil i2c-tools

   # Arch Linux/Manjaro
   sudo pacman -S ddcutil i2c-tools
   ```

## Development Setup

### Project Structure

```
twinkle-linux/
├── src/
│   ├── main.py              # Application entry point
│   ├── core/                # Core application logic
│   │   ├── app.py           # Application controller
│   │   ├── config.py        # Configuration management
│   │   └── logging.py       # Logging configuration
│   ├── ddc/                 # DDC/CI abstraction layer
│   │   ├── command.py       # DDC/CI command execution
│   │   ├── ddc_manager.py   # DDC/CI manager
│   │   ├── exceptions.py    # DDC/CI exceptions
│   │   ├── monitor.py       # Monitor model
│   │   └── vcp_codes.py     # VCP code definitions
│   ├── ui/                  # User interface layer
│   │   ├── ui_main.py       # Main UI controller
│   │   ├── brightness_popup.py  # Brightness popup
│   │   ├── widgets/         # UI widgets
│   │   │   ├── brightness_slider.py
│   │   │   ├── settings_dialog.py
│   │   │   └── vcp_controls.py
│   │   └── resources/       # UI resources
│   ├── services/            # Application services
│   └── utils/               # Utility functions
├── tests/                   # Test suite
│   ├── unit/                # Unit tests
│   ├── integration/         # Integration tests
│   └── fixtures/            # Test fixtures
├── packaging/               # Distribution packaging
├── scripts/                 # Utility scripts
├── docs/                    # Documentation
├── pyproject.toml           # Project configuration
├── requirements.txt         # Python dependencies
├── CHANGELOG.md             # Version history
├── CONTRIBUTING.md          # Contributing guidelines
├── LICENSE                  # MIT License
└── README.md                # Project overview
```

### Running the Application

```bash
# Run from source
python src/main.py

# Run with debug logging
python src/main.py --debug

# Run with custom config
python src/main.py --config /path/to/config.json
```

### Running Tests

```bash
# Run all tests
pytest

# Run with coverage
pytest --cov=src

# Run specific test file
pytest tests/unit/test_config.py

# Run with verbose output
pytest -v

# Run only unit tests
pytest tests/unit/

# Run only integration tests
pytest tests/integration/
```

## Code Style Guidelines

We follow these coding standards to maintain code quality and consistency:

### Python Style

- **PEP 8**: Follow Python Enhancement Proposal 8 style guide
- **Type Hints**: Use type hints for all function signatures and complex types
- **Docstrings**: Use Google-style docstrings for all public functions and classes
- **Line Length**: Maximum 100 characters per line

### Code Formatting

We use the following tools for code quality:

```bash
# Format code with Black
black src tests

# Check formatting without modifying
black --check src tests

# Run Pylint for linting
pylint src

# Run MyPy for type checking
mypy src

# Run isort for import sorting
isort src tests
```

### Naming Conventions

- **Classes**: `PascalCase` (e.g., `DDCManager`)
- **Functions/Methods**: `snake_case` (e.g., `get_brightness`)
- **Constants**: `UPPER_SNAKE_CASE` (e.g., `DEFAULT_TIMEOUT`)
- **Private Members**: `_leading_underscore` (e.g., `_internal_method`)
- **Protected Members**: `__double_underscore` (e.g., `__private_method`)

### Documentation

- All public functions and classes must have docstrings
- Use Google-style docstrings:
  ```python
  def get_brightness(monitor_id: str) -> int:
      """Get the current brightness level for a monitor.

      Args:
          monitor_id: The unique identifier of the monitor.

      Returns:
          The current brightness level (0-100).

      Raises:
          MonitorNotFoundError: If the monitor is not found.
          DDCError: If there's an error communicating with the monitor.
      """
      ...
  ```

## Commit Message Guidelines

We follow a conventional commit message format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `perf`: Performance improvements

### Examples

```
feat(ui): add brightness slider widget

Add a new brightness slider widget that allows users to
adjust brightness with a visual slider interface.

Closes #123
```

```
fix(ddc): handle monitor timeout errors

Add retry logic for DDC/CI commands that timeout after
the configured timeout period.

Fixes #456
```

## Pull Request Process

### Before Submitting

1. **Update Your Branch**

   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run Tests**

   ```bash
   pytest
   ```

3. **Run Code Quality Checks**

   ```bash
   black src tests
   pylint src
   mypy src
   ```

4. **Update Documentation**

   - Update relevant documentation files
   - Add docstrings to new functions/classes
   - Update CHANGELOG.md for user-facing changes

### Creating a Pull Request

1. Push your changes to your fork:

   ```bash
   git push origin feature/your-feature-name
   ```

2. Create a pull request on GitHub:
   - Go to the repository on GitHub
   - Click "New Pull Request"
   - Select your branch
   - Provide a descriptive title
   - Fill in the PR template

3. Wait for review and address any feedback

### Pull Request Checklist

- [ ] Code follows project style guidelines
- [ ] Tests pass locally
- [ ] New tests added for new features
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (for user-facing changes)
- [ ] Commit messages follow guidelines
- [ ] No merge conflicts with upstream/main

## Testing

### Writing Tests

We use pytest for testing. Tests should be organized as:

- **Unit Tests**: Test individual functions and classes in isolation
- **Integration Tests**: Test how components work together
- **Fixtures**: Reusable test data and setup

### Test Structure

```
tests/
├── unit/
│   ├── test_config.py
│   ├── test_ddc_manager.py
│   └── ...
├── integration/
│   ├── test_monitor_detection.py
│   └── ...
└── fixtures/
    ├── mock_monitor_data.py
    └── ...
```

### Example Test

```python
import pytest
from src.ddc.monitor import Monitor

def test_monitor_creation():
    """Test creating a new monitor instance."""
    monitor = Monitor(
        serial="LEN123456",
        name="Lenovo T24i",
        bus="i2c-3"
    )
    assert monitor.serial == "LEN123456"
    assert monitor.name == "Lenovo T24i"
    assert monitor.bus == "i2c-3"
```

### Running Tests

```bash
# Run all tests
pytest

# Run with coverage
pytest --cov=src --cov-report=html

# Run specific test
pytest tests/unit/test_config.py::test_load_config

# Run with verbose output
pytest -v

# Run only failed tests
pytest --lf
```

## Reporting Issues

When reporting bugs, please include:

1. **Environment Information**
   - Operating system and version
   - Python version
   - ddcutil version (`ddcutil --version`)
   - Twinkle Linux version

2. **Steps to Reproduce**
   - Clear, numbered steps to reproduce the issue
   - Expected behavior
   - Actual behavior

3. **Logs and Error Messages**
   - Relevant log output
   - Error messages
   - Screenshots (if applicable)

4. **Additional Context**
   - Monitor model and connection type
   - Any relevant configuration
   - Workarounds you've tried

Use the [GitHub Issues](https://github.com/twinkle-linux/twinkle-linux/issues) page to report bugs.

## Feature Requests

We welcome feature requests! When suggesting a new feature:

1. **Check Existing Issues**
   - Search for similar requests first
   - Add comments to existing requests if relevant

2. **Provide Context**
   - Describe the use case
   - Explain why this feature would be useful
   - Consider potential implementation approaches

3. **Be Specific**
   - Provide clear requirements
   - Consider edge cases
   - Think about user experience

4. **Consider Contributing**
   - If you're able to implement the feature, that's even better!
   - Open an issue first to discuss the approach

## Questions and Support

- **GitHub Discussions**: For general questions and discussions
- **GitHub Issues**: For bug reports and feature requests
- **Documentation**: Check the [docs/](docs/) directory for detailed guides

## Code of Conduct

Be respectful and constructive in all interactions. We're all here to improve Twinkle Linux together.

## License

By contributing to Twinkle Linux, you agree that your contributions will be licensed under the MIT License.

Thank you for contributing to Twinkle Linux!
