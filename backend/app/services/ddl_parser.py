import re
from typing import List, Dict, Tuple, Optional
from app.schemas.ddl import ParsedTable, ParsedColumn


def clean_identifier(name: str) -> str:
    """Clean quotes, schemas, and whitespaces from SQL identifiers."""
    if not name:
        return ""
    name = name.strip()
    # Strip quotes if wrapped
    if (name.startswith('"') and name.endswith('"')) or (name.startswith('`') and name.endswith('`')) or (name.startswith("'") and name.endswith("'")):
        name = name[1:-1]
    # If contains dot (e.g. SCHEMA.TABLE or TABLE.COLUMN), take the last part
    if "." in name:
        parts = [p.strip('"`\' ') for p in name.split(".")]
        name = parts[-1]
    return name.strip('"`\' ')


def extract_table_name_parts(name: str) -> Tuple[Optional[str], str]:
    """Extract optional schema and table name."""
    parts = [clean_identifier(p) for p in name.split(".")]
    if len(parts) >= 2:
        return parts[-2], parts[-1]
    return None, parts[0]


class DdlParser:
    @classmethod
    def parse(cls, sql_text: str, dialect: str = "dm") -> List[ParsedTable]:
        """
        Parse SQL DDL text into structured ParsedTable objects.
        Prioritizes Dameng (DM8/DM7), Oracle, MySQL, and PostgreSQL.
        """
        # Step 1: Extract all COMMENT ON TABLE and COMMENT ON COLUMN statements first
        table_comments: Dict[str, str] = {}
        column_comments: Dict[str, Dict[str, str]] = {}  # {table_name_upper: {column_name_upper: comment}}

        # Regex for COMMENT ON TABLE [schema.]table IS 'comment'
        comment_table_regex = re.compile(
            r"COMMENT\s+ON\s+TABLE\s+([`\"\w\.]+)\s+IS\s+'((?:''|[^'])*)'",
            re.IGNORECASE
        )
        for match in comment_table_regex.finditer(sql_text):
            tbl_raw, comment_val = match.groups()
            tbl_name = clean_identifier(tbl_raw).upper()
            table_comments[tbl_name] = comment_val.replace("''", "'")

        # Regex for COMMENT ON COLUMN [schema.]table.column IS 'comment'
        comment_col_regex = re.compile(
            r"COMMENT\s+ON\s+COLUMN\s+([`\"\w\.]+)\.([`\"\w]+)\s+IS\s+'((?:''|[^'])*)'",
            re.IGNORECASE
        )
        for match in comment_col_regex.finditer(sql_text):
            tbl_raw, col_raw, comment_val = match.groups()
            tbl_name = clean_identifier(tbl_raw).upper()
            col_name = clean_identifier(col_raw).upper()
            if tbl_name not in column_comments:
                column_comments[tbl_name] = {}
            column_comments[tbl_name][col_name] = comment_val.replace("''", "'")

        # Step 2: Extract CREATE TABLE statements
        tables: List[ParsedTable] = []
        
        # Regex to locate CREATE [GLOBAL TEMPORARY] TABLE [IF NOT EXISTS] name (...)
        create_table_regex = re.compile(
            r"CREATE\s+(?:GLOBAL\s+TEMPORARY\s+)?TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?([`\"\w\.]+)\s*\(",
            re.IGNORECASE
        )

        for match in create_table_regex.finditer(sql_text):
            tbl_raw = match.group(1)
            raw_table_name = clean_identifier(tbl_raw)
            tbl_name_upper = raw_table_name.upper()
            start_pos = match.end() - 1  # position of '('

            # Find matching closing parenthesis
            paren_count = 0
            in_single_quote = False
            in_double_quote = False
            end_pos = -1

            for i in range(start_pos, len(sql_text)):
                char = sql_text[i]
                if char == "'" and not in_double_quote:
                    in_single_quote = not in_single_quote
                elif char == '"' and not in_single_quote:
                    in_double_quote = not in_double_quote
                elif not in_single_quote and not in_double_quote:
                    if char == '(':
                        paren_count += 1
                    elif char == ')':
                        paren_count -= 1
                        if paren_count == 0:
                            end_pos = i
                            break

            if end_pos == -1:
                continue

            columns_block = sql_text[start_pos + 1:end_pos]
            after_block = sql_text[end_pos + 1:end_pos + 400]  # examine table options e.g. COMMENT='...'

            # Extract MySQL style table comment if present
            mysql_tbl_comment_match = re.search(
                r"COMMENT\s*=\s*'((?:''|[^'])*)'",
                after_block,
                re.IGNORECASE
            )
            table_comment = table_comments.get(tbl_name_upper, "")
            if not table_comment and mysql_tbl_comment_match:
                table_comment = mysql_tbl_comment_match.group(1).replace("''", "'")

            # Parse columns from columns_block
            parsed_columns = cls._parse_columns_block(columns_block, tbl_name_upper, column_comments)

            tables.append(ParsedTable(
                table_name=raw_table_name,
                table_comment=table_comment,
                columns=parsed_columns
            ))

        return tables

    @classmethod
    def _parse_columns_block(
        cls,
        columns_block: str,
        tbl_name_upper: str,
        column_comments: Dict[str, Dict[str, str]]
    ) -> List[ParsedColumn]:
        """Split and parse individual column definitions inside CREATE TABLE (...)"""
        lines: List[str] = []
        current_part = []
        paren_count = 0
        in_single_quote = False
        in_double_quote = False

        for char in columns_block:
            if char == "'" and not in_double_quote:
                in_single_quote = not in_single_quote
            elif char == '"' and not in_single_quote:
                in_double_quote = not in_double_quote
            elif not in_single_quote and not in_double_quote:
                if char == '(':
                    paren_count += 1
                elif char == ')':
                    paren_count -= 1
                elif char == ',' and paren_count == 0:
                    lines.append("".join(current_part).strip())
                    current_part = []
                    continue
            current_part.append(char)

        if current_part:
            lines.append("".join(current_part).strip())

        parsed_cols: List[ParsedColumn] = []
        sort_order = 1

        ignore_prefixes = (
            "PRIMARY KEY", "CONSTRAINT", "FOREIGN KEY", "KEY", "INDEX",
            "UNIQUE KEY", "UNIQUE INDEX", "CHECK", "FULLTEXT", "SPATIAL"
        )

        for line in lines:
            if not line:
                continue

            # Strip any trailing comma or spaces
            line = line.rstrip(",").strip()
            upper_line = line.upper()

            # Skip table-level constraints
            if any(upper_line.startswith(prefix) for prefix in ignore_prefixes):
                continue

            # Check for MySQL inline comment: COMMENT 'some comment'
            inline_comment = ""
            mysql_col_comment_match = re.search(
                r"\s+COMMENT\s+'((?:''|[^'])*)'",
                line,
                re.IGNORECASE
            )
            if mysql_col_comment_match:
                inline_comment = mysql_col_comment_match.group(1).replace("''", "'")
                # Remove COMMENT clause from line for cleaner datatype parsing
                line = line[:mysql_col_comment_match.start()] + line[mysql_col_comment_match.end():]

            # Extract column name and column data type
            # Column name might be quoted: "USER_ID" NUMBER(10, 2) NOT NULL...
            col_match = re.match(
                r"^\s*([`\"']?[a-zA-Z0-9_\u4e00-\u9fa5]+[`\"']?)\s+([a-zA-Z0-9_]+(?:\s*\([^\)]*\))?)",
                line
            )
            if not col_match:
                continue

            raw_col_name = clean_identifier(col_match.group(1))
            data_type = col_match.group(2).strip().upper()
            col_name_upper = raw_col_name.upper()

            # Determine comment: priority to COMMENT ON COLUMN statement, then inline COMMENT
            comment = inline_comment
            if tbl_name_upper in column_comments and col_name_upper in column_comments[tbl_name_upper]:
                comment = column_comments[tbl_name_upper][col_name_upper]

            parsed_cols.append(ParsedColumn(
                column_name=raw_col_name,
                data_type=data_type,
                column_comment=comment,
                sort_order=sort_order
            ))
            sort_order += 1

        return parsed_cols
