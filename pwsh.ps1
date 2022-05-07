New-Module -Script {
    function mydocker {
        [CmdletBinding()]
        Param
        (
             [parameter(mandatory=$false, ValueFromRemainingArguments=$true)] $rest
        )
        docker build -t mydocker . && docker run --cap-add="SYS_ADMIN" mydocker @rest
    }
    Export-ModuleMember -Function mydocker
}
