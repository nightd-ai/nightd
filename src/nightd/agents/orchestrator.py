"""Main orchestrator that coordinates the workflow between agents."""

import logging

from .spec_writer import write_spec
from .coder import implement_spec
from .reviewer import review_changes

logger = logging.getLogger(__name__)


async def run_workflow(task_description: str) -> dict:
    """Run the complete workflow from spec writing through review.

    Args:
        task_description: Description of the task to be completed.

    Returns:
        Dict containing spec, implementation, and review results.
        If any step fails, partial results are returned with error info.
    """
    logger.info("Starting workflow for task: %s", task_description[:50] + "...")

    result = {
        "spec": None,
        "implementation": None,
        "review": None,
    }

    try:
        logger.info("Step 1: Writing spec")
        spec = await write_spec(task_description)
        result["spec"] = spec
        logger.info("Spec created successfully")
    except Exception as e:
        logger.error("Spec writing failed: %s", e)
        result["spec"] = {"error": str(e)}
        return result

    try:
        logger.info("Step 2: Implementing spec")
        implementation_summary = await implement_spec(spec)
        result["implementation"] = implementation_summary
        logger.info("Implementation completed successfully")
    except Exception as e:
        logger.error("Implementation failed: %s", e)
        result["implementation"] = {"error": str(e)}
        return result

    try:
        logger.info("Step 3: Reviewing changes")
        review = await review_changes(spec, implementation_summary)
        result["review"] = review
        logger.info("Review completed successfully")
    except Exception as e:
        logger.error("Review failed: %s", e)
        result["review"] = {"error": str(e)}
        return result

    logger.info("Workflow completed successfully")
    return result
